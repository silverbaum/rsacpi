use std::{fs::{self, read_dir}, io, env};


fn bat(dir: &str) -> Result<String, io::Error>
{
    for entry_r in read_dir(dir)? {
        let entry = entry_r?;
        let file_namebuf = entry.file_name();
        let file_name = file_namebuf.to_str().unwrap_or("Error in to_str");

        if file_name.contains("BAT") || file_name.contains("bat") && entry.file_type()?.is_dir() 
        {
                //println!("Found {} with path {}", file_name, entry.path().to_string_lossy());
                let capacity = fs::read_to_string(format!("{}/capacity", entry.path().to_str().unwrap()))?;
                //let mut capbuf: [u8; 1] = [0; 1];
                //let _ = capacity.read(&mut capbuf);
                let cap: i32 = capacity.trim().parse().unwrap();
                let status = match fs::read_to_string(format!("{}/status", entry.path().to_string_lossy())) {
                Ok(str) => str, Err(_) => "Unknown".to_owned()};

                return Ok(String::from( format!("{file_name}: {}%, {}", cap, status) ));

        }
    }

    Err(io::Error::other("No batteries found"))
}


fn thermal(thermdir: &str) -> Result<(), io::Error>
{
    let mut found: bool = false;
    let mut entries = read_dir(thermdir)?
                    .map(|i| i.map(|e| e.path().to_owned()))
                    .collect::<Result<Vec<_>, io::Error>>()?;

    entries.sort();

   // let mut batteries = Vec::<String>::new;
    
    for f in entries.iter() {
        let file_path = f.to_str().unwrap_or("err: to_str");
        
        if file_path.contains("thermal_zone") {
                found = true;
                let file_name: &str = file_path.strip_prefix(format!("{thermdir}/thermal_zone").as_str()).unwrap();
                let temp: String = fs::read_to_string(format!("{}/temp", file_path))?;
                let tempi: f32 = match temp.trim().parse::<f32>() {
                    Ok(n) => n/1000.0,
                    Err(_) => 0.0
                };

                println!("Thermal {}: {:.1} C°", file_name, tempi);
        }
    }

    match found {
    true => Ok(()),
    false => Err(io::Error::other("No thermals found"))
    }
}


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

fn help() 
{
println!("rsacpi [OPTION]\n\n\
-b  Display Battery information\n\
-a  Display AC adapter information\n\
-t  Display thermal zone temperatures\n\
-A  Prints all available information\n\
-h  Display this help information and exit")
}


fn main()
{
    let batdir = "/sys/class/power_supply";
    
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        match bat(batdir) {
        Ok(str) => print!("{str}"),
        Err(e) => println!("{e}")
        }
        return;
    }

    if args[1].contains("b") {
        
        match bat(batdir) {
        Ok(str) => print!("{str}"),
        Err(e) => println!("{e}")
        }

    } else if args[1].contains("t") {
        thermal("/sys/class/thermal").unwrap_or_else(|e| println!("{e}"));
    } else if args[1].contains("a") {
        
        match ac("/sys/class/power_supply") {
        Ok(str) => println!("{str}"),
        Err(e) => println!("{e}")
        
        }
    } else if args[1].contains("A") {
        
        match bat(batdir) {
        Ok(str) => print!("{str}"),
        Err(e) => println!("{e}")
        }
        
        match ac(batdir) {
        Ok(str) => println!("{str}"),
        Err(e) => println!("{e}"),
        }

        thermal("/sys/class/thermal").unwrap_or_else(|e| println!("{e}"));

    } else if args[1].contains("h") {
        let _ = help();
    }

    
}
