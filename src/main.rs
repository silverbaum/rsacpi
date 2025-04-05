use std::{fs::{self, read_dir}, io, env};


fn bat(dir: &str) -> io::Result<()> 
{
    for entry_r in read_dir(dir)? {
        let entry = entry_r?;
        let file_namebuf = entry.file_name();
        let file_name = file_namebuf.to_str().unwrap();

        if file_name.contains("BAT") || file_name.contains("bat") && entry.file_type()?.is_dir() 
        {
                //println!("Found {} with path {}", file_name, entry.path().to_string_lossy());
                let capacity = fs::read_to_string(format!("{}/capacity", entry.path().to_str().unwrap()))?;
                //let mut capbuf: [u8; 1] = [0; 1];
                //let _ = capacity.read(&mut capbuf);
                let cap: i32 = capacity.trim().parse().unwrap();
                
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
                let tempi: i32 = match temp.trim().parse() {
                    Ok(n) => n,
                    Err(_) => 0
                };
                temp = (tempi / 1000).to_string();

                println!("Thermal {}: {}C", file_name, temp);
        }
    }

    Ok(())
}

fn ac(acdir: &str) -> io::Result<()> {
      for entry_r in read_dir(acdir)? {
        let entry = entry_r?;
        let file_namebuf = entry.file_name();
        let file_name = file_namebuf.to_str().unwrap_or_default();
        
        if file_name.contains("AC") || file_name.contains("ac") && entry.file_type()?.is_dir() 
        {
                let ac_online = fs::read_to_string(format!("{}/online", entry.path().to_str().unwrap()))?;
                let ac_status: i32 = ac_online.trim().parse().unwrap();

                let ac_str: &str;
                if ac_status == 1 {
                    ac_str = "online";
                } else {
                    ac_str = "off-line";
                }

                print!("{file_name}: {}\n", ac_str);
        }
    }

    Ok(())
}

fn main() {
    let batdir = "/sys/class/power_supply";
    
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        let _ = bat(batdir);
        return;
    }

    if args[1].contains("b") {
        let _ = bat(batdir);
    } else if args[1].contains("t") {
        let _ = thermal("/sys/class/thermal");
    } else if args[1].contains("a") {
        let _ = ac("/sys/class/power_supply");
    } else if args[1].contains("A") {
        let _ = bat(batdir);
        let _ = ac(batdir);
        let _ = thermal("/sys/class/thermal");
    }

    
}
