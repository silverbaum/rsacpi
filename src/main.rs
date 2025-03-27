use std::{fs::{self, File}, io};
use std::io::prelude::*;


fn bat(dir: &str) -> io::Result<()>
{
let entries = fs::read_dir(dir);

//println!("{:?}", &entries.unwrap());
let entr = entries.unwrap()
    .map(|i| i.map(|e| e.file_name().into_string().unwrap()))
    .collect::<Result<Vec<String>, io::Error>>();


for bat in entr? {
    let mut capf = File::open(format!("{}/{}/online", dir, bat))?;
    let mut capbuf = [0; 1];
    let _ =  capf.read(&mut capbuf)?;

    let statf = File::open(format!("{}/{}/status", dir, bat))?;
    let mut statreader = io::BufReader::new(statf);
    let mut statbuf: String = String::new();
    statreader.read_line(&mut statbuf)?;


    println!("{}: {:?}%, {}", bat, &capbuf[0], statbuf);

}



Ok(())

}

fn main() -> io::Result<()> {
    let batdir = "/sys/class/power_supply";
    let _ = bat(batdir);
    //println!("{:?}", entries?);   


    Ok(())
}
