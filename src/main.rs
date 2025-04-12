//Copyright (C) 2025 Topias Silfverhuth
//SPDX-License-Identifier: MIT

use std::{fs::{self, read_dir}, io};
 
struct Args {
///Display battery information
battery: bool,
///Display AC adapter information
ac: bool,
///Display thermal information
thermal: bool,
///Display all the available information
everything: bool,
///help me
help: bool,
}

static PWDIR: &str = "/sys/class/power_supply";
static THDIR: &str = "/sys/class/thermal";

///Displays battery information
fn bat(dir: &str) -> Result<String, io::Error>
{
        
    //let mut fields: Vec<Batf> = vec![ Batf {field:"capacity", info:"".to_owned()}, Batf {field: "status", info:"".to_owned()}];

    let entries: Vec<_> = read_dir(dir)?
                .map(|e| {e.unwrap_or_else(|er| panic!("{er}"))})
                .filter(|entry| {
                    entry.file_name().to_string_lossy().contains("BAT") 
                 || entry.file_name().to_string_lossy().contains("bat")
                })
                .collect();

        
    for entry in entries.iter() {
        let file_namebuf = entry.file_name();
        let file_name = file_namebuf.to_str().expect("to_str");

        let capacity_r = fs::read_to_string(format!("{}/capacity", entry.path().to_str().expect("to_str")))?;
        let capacity: i32 = capacity_r.trim().parse().expect("Failed to convert to an integer");
        
        let status_r = match fs::read_to_string(format!("{}/status", entry.path().to_string_lossy())) {
                        Ok(str) => str,
                        Err(_) => "Unknown".to_owned()
                        };
        let status: &str = status_r.trim();

        let energy_r = fs::read_to_string(format!("{}/energy_now", entry.path().to_str().expect("to_str: energy_now not found")))?;
        let energy: i64 = energy_r.trim().parse().expect("energy_now should be an integer");
            
        let power_r = fs::read_to_string(format!("{}/power_now", entry.path().to_str().unwrap_or("oh no")))?;
        let power: i64 = power_r.trim().parse().expect("power_now should be an integer");
        
        if energy > 0 && power > 0 && status == "Discharging" {

            let seconds: i64 = 3600 * energy / power;
            if seconds > 0 {
                let hours = seconds / 3600;
                let minutes = (seconds % 3600) / 60;
    
                let remaining = format!("{}h {}m", hours, minutes);
                return Ok(String::from( format!("{file_name}: {}%, {}, {} remaining", capacity, status, remaining) ));
            }
        }

        return Ok(String::from( format!("{file_name}: {}%, {}", capacity, status) ));
    }

    Err(io::Error::other("No batteries found"))
}

///iterates through thermdir and prints the temperature of all thermal zones
fn thermal(thermdir: &str) -> Result<(), io::Error>
{
    let mut found = false;


    let mut entries = read_dir(thermdir)?
                    .map(|i| i.map(|e| e.path().to_owned()))
                    .collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort();

    let files = entries.iter().map(|f| f.to_str().unwrap_or("err: to_str"))
                .filter(|path| path.contains("thermal_zone"))
                .collect::<Vec<&str>>();
                    
    for f in files.iter() {
        let num = f.strip_prefix(format!("{thermdir}/thermal_zone").as_str()).unwrap();
        let temp: String = fs::read_to_string(format!("{}/temp", f))?;
        let tempi: f32 = match temp.trim().parse::<f32>() {
        Ok(n) => n/1000.0,
        Err(_) => 0.0
        };
        
        println!("Thermal {}: {:.1} C°", num, tempi);
        found = true;
    }
    
    match found{
    true => Ok(()),
    false => Err(io::Error::other("No thermals found"))
    }
}

///Prints AC adapter information to stdout
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
                let ac_online = fs::read_to_string(format!("{}/online", entry.path().to_str().unwrap()))?;
                let ac_status: i32 = ac_online.trim().parse().unwrap();

                let ac_str: &str;
                match ac_status {
                    1 => ac_str = "on-line",
                    0 => ac_str = "off-line",
                    _ => ac_str = "Unknown",
                }

                return Ok(String::from(format!("{file_name}: {}", ac_str)));
            }
            }
    
        Err(_) => return no_ac
      }
      }

      no_ac
}


fn usage() {
println!("Usage: rsacpi [OPTION]...\n\n\
-b, --battery       Displays battery information\n\
-t, --thermal       Displays temperatures from all thermal zones\n\
-a, --ac            Displays AC adapter status\n\
-e, --everything    Displays all available information\n\
-h, --help          Displays this help information\n");
}

///poor argument reaping and sowing embolism (parse)
fn parse() -> Args {

    let mut ac: bool = false;
    let mut battery: bool = false;
    let mut thermal: bool = false;
    let mut everything: bool = false;
    let mut help: bool = false;
    
    
    for arg in std::env::args() {
        if arg.contains("--") {
            if arg.contains("help"){
                help = true;
            }
        } 
        else if arg.contains("-")  {
            if arg.contains("a") {
                ac = true;
            } else if arg.contains("b") {
                battery = true;
            } else if arg.contains("t") {
                thermal = true;
            } else if arg.contains("e") {
                everything = true;
            } else if arg.contains("h") {
                help = true;
            } else {
                println!("Unknown argument {arg}");
                help = true;
            }
        }
            }


    let opts = Args {battery:battery, thermal:thermal, ac:ac,
                     everything:everything, help:help};
    opts
}


fn main()
{
       
    let mut args = parse();

    if args.help {
        usage();
        return;
    }
    
    if args.everything {
        args.battery = true; args.ac = true; args.thermal = true;
    }

    if args.battery ||
    (!args.battery && !args.ac && !args.everything && !args.thermal){
        match bat(PWDIR) {
        Ok(str) => println!("{str}"),
        Err(e) => println!("{e}")
        }
    }
    if args.ac {
        match ac(PWDIR) {
        Ok(str) => println!("{str}"),
        Err(e) => println!("{e}")
        }
    }
    if args.thermal {
        thermal(THDIR).unwrap_or_else(|e| println!("{e}"));
    }
    
    

    
    

    
}



#[cfg(test)]
mod tests {
use super::*;

    #[test]
    fn battery() {
        let msg = bat(PWDIR).unwrap();
        assert!(msg.contains("%"));
    }

    #[test]
    fn battery_err() {

        let result =  bat("/sys/class/thermal");

        println!("{:?}", result);
        let e = match result {
        Ok(str) => format!("{str}"),
        Err(_) => format!("Error")
        };
        
        
        assert_eq!(e, "Error");
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
        Ok(_) => "passed",
        Err(_) => "failed"
        };

        assert_eq!(e, "passed")
    }

    #[test]
    fn thermals_err() {
        let result = thermal(PWDIR);

        let e = match result {
        Ok(_) => "failed",
        Err(_) => "passed"
        };

        assert_eq!(e, "passed")
    }



}





