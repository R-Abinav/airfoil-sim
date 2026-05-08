use std::fs;
use std::path::Path;
use crate::airfoil::{Airfoil, Point};

/*
NOTE: Lednicer format has not been implemented yet! Will be done soon!
*/

pub fn load_dat(path: &Path) -> Result<Airfoil, String>{
    let raw = fs::read_to_string(path)
                    .map_err(|err| format!("Could not read file {:?}: {}", path, err));
    
    let mut lines = raw.lines();

    //first line is always the airfoil name
    let name = lines.next().ok_ok("Empty_file").trim().to_string();

    let mut points: Vec<Point> = Vec::new();

    for line in lines{
        let line = line.trim();

        //skip blank and comment lines
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }

        //split on whitespace, parse it as two floats
        let mut parts = line.split_whitespace();

        let x = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok());
        let y = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok());

        match (x, y) {
            (Some(x), Some(y)) => points.push(Point { x, y }),
            _ => continue, //skip malformed lines silently
        }
    }

    if points.len() < 3{
            return Err(format!("Too few points in {:?}", path));
    }

    Ok(Airfoil { name, points })
}

//load all .dat files present in data/airfoils
pub fn load_all(dir: &Path) -> Vec<Airfoil>{
    let mut airfoils = Vec::new();

    let entries = match fs::read_dir(dir){
        Ok(e) => e,
        Err(_) => {
            eprintln!("Could not read directory {:?}", dir);
            return airfoils;
        }
    };

    //unwrap it using flatten
    for entry in entries.flatten(){
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("dat"){
            match load_dat(&path) {
                Ok(af) => {
                    println!("Loaded: {}", af.name);
                    airfoils.push(af);
                }
                Err(e) => eprintln!("Skipped {:?}: {}", path, e),
            }
        }
    }

    airfoils
}