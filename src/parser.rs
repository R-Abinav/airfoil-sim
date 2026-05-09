use std::fs;
use std::path::Path;
use crate::airfoil::{Airfoil, Point};

//parse a single "x y" coordinate line. Returns None if the line doesn't
//contain exactly two floats (used to distinguish the Lednicer count line).
fn parse_xy(line: &str) -> Option<(f64, f64)> {
    let mut parts = line.split_whitespace();
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    
    //a third token means it's not a plain coord line — reject it.
    if parts.next().is_some() { return None; }
    Some((x, y))
}

pub fn load_dat(path: &Path) -> Result<Airfoil, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("Could not read file {:?}: {}", path, err))?;

    let mut line_iter = raw.lines().peekable();

    //first line is always the airfoil name.
    let name = line_iter.next().ok_or("Empty file".to_string())?.trim().to_string();

    //collect remaining non-blank, non-comment lines as trimmed strings.
    let data_lines: Vec<&str> = line_iter
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#') && !t.starts_with('!')
        })
        .collect();

    //detect Lednicer format: first data line is the count line "N_upper  N_lower"
    //where both values are > 2.0 (real coordinates are always 0 ≤ x ≤ 1 after
    //normalization, but count lines have values like 61.0).
    let is_lednicer = if let Some(first) = data_lines.first() {
        if let Some((a, b)) = parse_xy(first) {
            a > 2.0 && b > 2.0
        } else {
            false
        }
    } else {
        false
    };

    let points: Vec<Point> = if is_lednicer {
        //skip the count line; the rest alternate upper block / blank / lower block.
        //since we already stripped blanks, we just split on the duplicate x=0 point
        //that resets after the upper surface ends and lower surface starts.
        //strategy: collect all parsed (x,y) pairs (skip count line), then split
        //into upper and lower at the second occurrence of x ≈ 0.0.
        let coord_lines = &data_lines[1..]; //drop count line

        let mut all: Vec<Point> = coord_lines
            .iter()
            .filter_map(|l| parse_xy(l.trim()).map(|(x, y)| Point { x, y }))
            .collect();

        //find the index of the second point with x ≈ 0 (start of lower surface).
        let split_idx = all
            .iter()
            .enumerate()
            .skip(1) //skip the very first point (start of upper surface)
            .find(|(_, p)| p.x < 1e-6)
            .map(|(i, _)| i)
            .ok_or_else(|| format!("Lednicer: could not find lower surface start in {:?}", path))?;

        let lower = all.split_off(split_idx); //lower surface: LE → TE
        let upper = all;                       //upper surface: LE → TE

        //build a closed contour going TE → upper → LE → lower → TE.
        //upper reversed gives TE → LE, lower forward gives LE → TE.
        let mut contour: Vec<Point> = upper.into_iter().rev().collect();
        contour.extend(lower.into_iter().skip(1)); // skip duplicate LE point
        contour
    } else {
        //selig format: coordinates already form a closed contour.
        data_lines
            .iter()
            .filter_map(|l| parse_xy(l.trim()).map(|(x, y)| Point { x, y }))
            .collect()
    };

    if points.len() < 3 {
        return Err(format!("Too few points in {:?}", path));
    }

    //normalise chord to 1.0 — runs for BOTH Selig and Lednicer paths.
    let x_max = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let x_min = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let chord  = x_max - x_min;

    let mut points = points;
    if chord > 0.0 {
        for p in &mut points {
            p.x = (p.x - x_min) / chord;
            p.y =  p.y          / chord;
        }
    }

    println!("  chord detected: {:.4}, points: {} ({})",
        chord, points.len(), if is_lednicer { "Lednicer" } else { "Selig" });

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