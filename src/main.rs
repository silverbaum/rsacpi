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
            let mut capacity = File::open(format!("{}/capacity", entry.path().to_str().unwrap()))?;
            let mut capbuf: [u8; 1] = [0; 1];
            let _ = capacity.read(&mut capbuf);
            
            let status = fs::read_to_string(format!("{}/status", entry.path().to_string_lossy()))?;
            print!("{file_name}: {}%, {}", capbuf[0], status);

        }
    }

    Ok(())
}


fn thermal(thermdir: &str) -> io::Result<()>
{
    let mut entries = read_dir(thermdir)?.map(|i| i.map(|e| e.path().to_owned())).collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort();
    
    //println!("{:?}", entries[0]);
    for f in entries {
        //let file_name = f.file_name().into_string().unwrap();
        let file_path = f.to_str().unwrap();
        
        
        if file_path.contains("thermal_zone"){
                let file_name = file_path.strip_prefix(format!("{thermdir}/thermal_zone").as_str()).unwrap();
                //let mut temp = File::open(format!("{}/temp", file_path))?;
                //let mut tempbuf: [u8; 5] = [0; 5];
                //let _ = temp.read(&mut tempbuf);
                let mut temp = fs::read_to_string(format!("{}/temp", file_path))?;
                let mut tempi: i32 = temp.trim().parse().unwrap();
                temp = (tempi / 1000).to_string();

                println!("Thermal {}: {}C", file_name, temp);
        }
    }
   
/*
    for entry_r in read_dir(thermdir)? {
        let entry = entry_r?;
        let file_namebuf = entry.file_name();
        let file_name = file_namebuf.to_str().unwrap();

        //println!("{:?}, file_name: {}", entry.path(), file_name);

        if file_name.contains("thermal")
        {
            //println!("Found {} with path {}", file_name, entry.path().to_string_lossy());
            let mut temp = File::open(format!("{}/temp", entry.path().to_str().unwrap()))?;
            let mut tempbuf: [u8; 1] = [0; 1];
            let _ = temp.read(&mut tempbuf);

            println!("{file_name}: {}C", tempbuf[0]);

        }
    }
*/

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
