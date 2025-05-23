//Copyright (C) 2025 Topias Silfverhuth
//SPDX-License-Identifier: MIT

use std::{
    fs::{read_dir, read_to_string, read}, 
    process::exit,
    io
};

const VERSION: &str = "0.4.3";

struct Args {
///battery information
battery: bool,
///Battery health information
battery_health: bool,
/// AC adapter information
ac: bool,
/// thermal information
thermal: bool,
/// all the available information
everything: bool,
///help me
help: bool,
///Version and license information
version: bool
}

const PWDIR: &str = "/sys/class/power_supply";
const THDIR: &str = "/sys/class/thermal";


fn battery_design(dir: &str) -> Result<Vec<String>, io::Error> 
{
    let mut batteries: Vec<String> = Vec::new();

    let entries: Vec<_> = read_dir(dir)?
                .map(|e| {e.unwrap_or_else(|er| panic!("{}", er))})
                .filter(|entry| {
                    entry.file_name().to_string_lossy().contains("BAT") 
                 || entry.file_name().to_string_lossy().contains("bat")
                })
                .collect();
    for entry in entries.iter() {

        let fname = entry.file_name();
        let file_name = fname.to_str().unwrap();

        let voltage_r = match read_to_string(format!("{}/voltage_now", entry.path().to_string_lossy())) {
            Ok(str) => str,
            Err(_e) => "Err".to_owned()
        };
        let voltage: i128 = match voltage_r.trim().parse::<i128>() {
            Ok(int) => int / 100000, //from uV, 10⁵ -> centivolts
            Err(_e) => -1
        };

        
        let design_capr = match read_to_string(format!("{}/energy_full_design", entry.path().to_string_lossy())) {
            Ok(str) => str,
            Err(_) => "Unknown".to_owned()
            };
        let design_cap: i128 = match design_capr.trim().parse::<i128>() {
            Ok(int) => {
                if voltage != -1 {int/100 / (voltage)} else {int/1000} //convert from uWh to mAh if voltage is found, otherwise convert to mWh
            },
            Err(e) => return Err(io::Error::other(format!("Unable to determine battery's design capacity: {e}")))
        };

        let real_capr = match read_to_string(format!("{}/energy_full", entry.path().to_string_lossy())) {
            Ok(str) => str,
            Err(_) => "Unknown".to_owned()
            };
        let real_cap: i128 = match real_capr.trim().parse::<i128>() {
            Ok(int) => {if voltage != -1 {int/100 / voltage} else {int/1000}},
            Err(e) => return Err(io::Error::other(format!("Unable to determine battery capacity: {e}")))
        };
        let battery_health : f64 = (<i32 as Into<f64>>::into(real_cap as i32) / <i32 as Into<f64>>::into(design_cap as i32)) * 100.0;
        
        let unit = if voltage != -1 {"mAh"} else {"mWh"};
        batteries.push(format!("{file_name}: {real_cap} / {design_cap} {unit} = {:.1}%", battery_health));
    }

    Ok(batteries)
}

///Displays battery information
fn bat(dir: &str) -> Result<Vec<String>, io::Error>
{
    let mut batteries: Vec<String> = Vec::new();
    let mut found = false;

    let entries: Vec<_> = read_dir(dir)?
                .map(|e| {e.unwrap_or_else(|er| panic!("{}", er))})
                .filter(|entry| {
                    entry.file_name().to_string_lossy().contains("BAT") 
                 || entry.file_name().to_string_lossy().contains("bat")
                })
                .collect();

        
    for entry in entries.iter() {
        let file_namebuf = entry.file_name();
        let file_name = file_namebuf.to_str().unwrap();

        let cap_path = format!("{}/capacity", entry.path().to_str().unwrap_or("/sys/class/power_supply/BAT0"));
        let capacity = 
        if let Ok(capacity_r) = read_to_string(&cap_path){
            capacity_r.trim().parse().unwrap_or(-1)
        } else {
            -1
        };
        
        
        let status_r = read_to_string(format!("{}/status", entry.path().to_string_lossy())).unwrap_or("Unknown".to_string()); 
        let status = status_r.trim();
        

        //read raw charge and power consumption for time estimate
        let energy_path = format!("{}/energy_now", entry.path().to_str().expect("Failed to convert path to string"));
        let energy = if let Ok(energy_r) = read_to_string(&energy_path) {
            
            match energy_r.trim().parse() {
                Ok(int) => int,
                Err(_) => {
                    match read(&energy_path) {
                        Ok(reader) => {
                            let res: i64 = unsafe {(*reader.align_to::<i64>().1)[0]};
                            res
                        },
                        Err(_) => -1
                    }
                }
            }
        } else {
            -1
        };

        

        let power = if let Ok(power_r) = read_to_string(format!("{}/power_now", entry.path().to_str().unwrap_or("oh no"))) {
    
        
            if power_r.is_empty() {
                -1
            } else {
                power_r.trim().parse().unwrap_or(-1)
            }
        } else {
            -1
        };
        

        //shows estimate of time remaining
        if energy > 0 && power > 0 && capacity >= 0 && status == "Discharging" {

            let seconds: i64 = 3600 * energy / power;
            if seconds > 0 {
                let hours = seconds / 3600;
                let minutes = (seconds % 3600) / 60;
    
                let remaining = format!("{}h {}m", hours, minutes);
                batteries.push(format!("{file_name}: {}%, {}, {} remaining", capacity, status, remaining) );
            }
        } else if capacity >= 0 && status != "Unknown" {
            batteries.push(format!("{file_name}: {}%, {}", capacity, status) )
        } else if capacity >= 0 {
                batteries.push(format!("{file_name}: {}%, {}", capacity, status) );
        } else {
                if let Ok(reader) = read(&cap_path) {
                    let capstr = String::from_utf8(reader).unwrap_or(String::from(""));
                    if !capstr.is_empty() {
                        batteries.push(format!("{file_name}: {}%, {}", capstr.trim(), status));
                    }
                } else {
                    return Err(io::Error::other("No battery information"));
                } 
                

        }
        
        
        found = true;
    }

    match found {
    true => Ok(batteries),
    false => Err(io::Error::other("No batteries found"))
    }
}

///Thermal zones
fn thermal(thermdir: &str) -> Result<Vec<String>, io::Error>
{
    let mut found = false;

    let mut temps: Vec<String> = Vec::new();


    let mut entries = read_dir(thermdir)?
                    .map(|i| i.map(|e| e.path().to_owned()))
                    .collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort();

    let files = entries.iter().map(|f| f.to_str().unwrap_or("err: to_str"))
                .filter(|path| path.contains("thermal_zone"))
                .collect::<Vec<&str>>();
                    
    for f in files.iter() {
        let num = f.strip_prefix(format!("{thermdir}/thermal_zone").as_str()).unwrap();
        let temp: String = read_to_string(format!("{}/temp", f))?;
        let tempi: f32 = match temp.trim().parse::<f32>() {
        Ok(n) => n/1000.0,
        Err(_) => 0.0
        };
        
        temps.push(format!("Thermal {}: {:.1} C°", num, tempi));
        found = true;
    }
    
    match found{
    true => Ok(temps),
    false => Err(io::Error::other("No thermals found"))
    }
}

///Prints AC adapter information to stdout
///
///acdir: The name of the directory which contains the AC adapter
fn ac(acdir: &str) -> Result<String, io::Error> 
{
    let no_ac: Result<String, io::Error> = Err(io::Error::other("No AC adapter found"));

    for entry_r in read_dir(acdir)?     {

      match entry_r {
        Ok(ent) => {
            let entry = ent; 
            let file_namebuf = entry.file_name();
            let file_name = file_namebuf.to_str().unwrap_or("Error");

            if file_name.contains("AC") || file_name.contains("ac") && entry.file_type()?.is_dir() 
            {
                let ac_online = read_to_string(format!("{}/online", entry.path().to_str().unwrap()))?;
                let ac_status: i32 = ac_online.trim().parse().unwrap();

                let ac_str: &str = match ac_status {
                    1 => "on-line",
                    0 => "off-line",
                    _ => "Unknown",
                };

                return Ok(format!("{file_name}: {}", ac_str));
            }
            }
    
        Err(_) => return no_ac
      }
      }

      no_ac
}

fn usage() {
    println!("Usage: rsacpi [OPTION]...\n\n\
-b, --battery       battery information\n\
-i, --health        battery health\n\
-t, --thermal       temperatures from all thermal zones\n\
-a, --ac            AC adapter status\n\
-e, --everything    all available information\n\
-h, --help          command-line options\n\
-v, --version       version and license information");
    exit(1);
}

fn version() {
println!("rsacpi {VERSION}\n\
a simple tool to display battery, AC, and thermal info\n\
Copyright (C) 2025 Topias Silfverhuth\n\
License: MIT <https://opensource.org/license/mit>\n\
This is free software: you are free to change and redistribute it.\n\
There is NO WARRANTY, to the extent permitted by law.\n\
Report bugs at: <https://github.com/silverbaum/rsacpi>");
}

///poor argument reaping and sowing embolism (parse)
fn parse() -> Args 
{

    let mut ac: bool = false;
    let mut battery: bool = false;
    let mut battery_health: bool = false;
    let mut thermal: bool = false;
    let mut everything: bool = false;
    let mut help: bool = false;
    let mut version: bool = false;
    
    for arg in std::env::args() {
        if arg.contains("--") && arg.starts_with("--") {
            if arg.contains("help"){
                help = true;
            } else if arg.contains("battery") {
                battery = true;
            } else if arg.contains("thermal") {
                thermal = true;
            } else if arg.contains("everything") {
                everything = true;
            } else if arg.contains("ac") {
                ac = true;
            } else if arg.contains("version") {
                version = true;
            } else if arg.contains("health") {
                battery_health = true;
            } else {
                usage();
            }
        } else if arg.contains("-") && arg.starts_with("-") && !arg.starts_with("--") {
            let opts = arg.as_bytes();
            for opt in opts {
                match opt {
                    b'a' => ac=true,
                    b'b' => battery=true,
                    b't' => thermal=true,
                    b'e' => everything=true,
                    b'h' => help=true,
                    b'v' => version=true,
                    b'i' => battery_health=true,
                    b'-' => (),
                    _ => usage()
                }
            }
        }
            }


    Args {battery, battery_health, ac, thermal, everything, help, version}
}


fn main() -> io::Result<()>
{
    let mut args = parse();

    if args.help {
            usage();
            return Ok(());
    } else if args.version {
            version();
            return Ok(());
    }
    
    if args.everything {
            args.battery = true; args.battery_health = true; args.ac = true; args.thermal = true;
    }

    if args.battery || !args.ac && !args.everything && !args.thermal && !args.battery_health {
            match bat(PWDIR) {
            Ok(str) => {
                for bat in str {
                        println!("{bat}");
                }},
            Err(e) => println!("{e}")
            }
    }
    if args.battery_health {
            if let Ok(vec) = battery_design(PWDIR) { for bat in vec {
                println!("{bat}");
            } }
            
    }

    if args.ac {
            match ac(PWDIR) {
            Ok(str) => println!("{str}"),
            Err(e) => println!("{e}")
        }
    }
    if args.thermal {
            match thermal(THDIR) {
            Ok(temps) => {
                for temp in temps {
                    println!("{temp}");
                }
            },
            Err(e) => eprintln!("{e}")
            }
    }
    
    Ok(())
}



#[cfg(test)]
mod tests {
use super::*;

    #[test]
    fn battery() {
        let mut pc: usize = 0;
        let msg = bat(PWDIR).unwrap();
        for bat in &msg {
            if bat.contains("%") {
                pc += 1;
            }
        }
        assert!(pc == msg.len());
    }

    #[test]
    fn battery_err() {

        let result =  bat("/sys/class/thermal");

        println!("{:?}", result);
        let e = match result {
        Ok(_) => false,
        Err(_) => true
        };
        
        assert!(e);
    }

    #[test]
    fn adapter() {
        ac(PWDIR).unwrap();
    }

    #[test]
    fn adapter_err() {
        let result =  ac("/sys/class/thermal");

        println!("{result:?}");

        let e = match result {
        Ok(str) => format!("{str}"),
        Err(_) => format!("Error")
        };

        assert_eq!("Error", e);
    }

    #[test]
    fn thermals() {
        let result = thermal(THDIR);
        
        let e = match result {
        Ok(_) => true,
        Err(_) => false
        };

        assert!(e)
    }

    #[test]
    fn thermals_err() {
        let result = thermal(PWDIR);

        let e = match result {
        Ok(_) => false,
        Err(_) => true
        };

        assert!(e)
    }

    #[test]
    fn tst_main() {
        let result = main();
        assert!(result.is_ok())
    }

    #[test]
    fn tst_health() {
        let result = battery_design(PWDIR);
        
        let e = match result {
            Ok(vec) => {if vec[0].contains("mAh") && vec[0].contains("%") {
                true
            } else {
                false
            }},
            Err(_) => false
        };
        assert!(e)
    }



}





