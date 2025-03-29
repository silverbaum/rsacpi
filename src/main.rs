#![allow(unused)]
use std::{fs::{self, File, read_dir}, io, env};
use std::io::prelude::*;


fn bat(dir: &str) -> io::Result<()> 
{
    for entry_r in read_dir(dir)? {
        let entry = entry_r?;
        let file_namebuf = entry.file_name();
        let file_name = file_namebuf.to_str().unwrap();

        if file_name.contains("BAT") || file_name.contains("bat") && entry.file_type()?.is_dir() 
        {
                //println!("Found {} with path {}", file_name, entry.path().to_string_lossy());
                let mut capacity = fs::read_to_string(format!("{}/capacity", entry.path().to_str().unwrap()))?;
                //let mut capbuf: [u8; 1] = [0; 1];
                //let _ = capacity.read(&mut capbuf);
                let mut cap: i32 = capacity.trim().parse().unwrap();
                
                let status = fs::read_to_string(format!("{}/status", entry.path().to_string_lossy()))?;
                print!("{file_name}: {}%, {}", cap, status);

        }
    }

    Ok(())
}


fn thermal(thermdir: &str) -> io::Result<()>
{
    let mut entries = read_dir(thermdir)?
                    .map(|i| i.map(|e| e.path().to_owned()))
                    .collect::<Result<Vec<_>, io::Error>>()?;

    entries.sort();
    
    for f in entries {
        let file_path = f.to_str().unwrap();
        
        
        if file_path.contains("thermal_zone"){
                let file_name = file_path.strip_prefix(format!("{thermdir}/thermal_zone").as_str()).unwrap();
                let mut temp = fs::read_to_string(format!("{}/temp", file_path))?;
                let mut tempi: i32 = temp.trim().parse().unwrap();
                temp = (tempi / 1000).to_string();

                println!("Thermal {}: {}C", file_name, temp);
        }
    }

    Ok(())
}

fn main() {
    let batdir = "/sys/class/power_supply";
    
    let args: Vec<String> = env::args().collect();
    if(args.len() == 1) {
        bat(batdir);
        return;
    }

    if args[1].contains("b") {
        let _ = bat(batdir);
    } else if args[1].contains("t") {
        let _ = thermal("/sys/class/thermal");
    } else if args[1].contains("A") {
        let _ = bat(batdir);
        let _ = thermal("/sys/class/thermal");
    }

    
}
