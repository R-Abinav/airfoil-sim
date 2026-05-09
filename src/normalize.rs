use std::env;
use std::fs;
use std::process;

struct Point {
    x: f64,
    y: f64,
}

fn parse_xy(line: &str) -> Option<(f64, f64)> {
    let mut parts = line.split_whitespace();
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    if parts.next().is_some() { return None; }
    Some((x, y))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.dat> <output.dat>", args[0]);
        process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let raw = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading input file '{}': {}", input_path, err);
            process::exit(1);
        }
    };

    let mut line_iter = raw.lines().peekable();

    let name = match line_iter.next() {
        Some(line) => line.trim().to_string(),
        None => {
            eprintln!("Error: Input file '{}' is empty", input_path);
            process::exit(1);
        }
    };

    let data_lines: Vec<&str> = line_iter
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#') && !t.starts_with('!')
        })
        .collect();

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
        let coord_lines = &data_lines[1..];
        let mut all: Vec<Point> = coord_lines
            .iter()
            .filter_map(|l| parse_xy(l.trim()).map(|(x, y)| Point { x, y }))
            .collect();

        let split_idx = match all
            .iter()
            .enumerate()
            .skip(1)
            .find(|(_, p)| p.x < 1e-6)
            .map(|(i, _)| i)
        {
            Some(idx) => idx,
            None => {
                eprintln!("Error: Lednicer format could not find lower surface start in '{}'", input_path);
                process::exit(1);
            }
        };

        let lower = all.split_off(split_idx);
        let upper = all;

        let mut contour: Vec<Point> = upper.into_iter().rev().collect();
        contour.extend(lower.into_iter().skip(1));
        contour
    } else {
        data_lines
            .iter()
            .filter_map(|l| parse_xy(l.trim()).map(|(x, y)| Point { x, y }))
            .collect()
    };

    if points.is_empty() {
        eprintln!("Error: No valid coordinate points found in '{}'", input_path);
        process::exit(1);
    }

    let x_max = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let x_min = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let chord = x_max - x_min;

    if chord <= 0.0 {
        eprintln!("Error: Invalid chord length computed ({}), points might be malformed.", chord);
        process::exit(1);
    }

    let mut points = points;
    for p in &mut points {
        p.x = (p.x - x_min) / chord;
        p.y = p.y / chord;
    }

    let mut output_content = format!("{}\n", name);
    for p in &points {
        output_content.push_str(&format!("{:.6} {:.6}\n", p.x, p.y));
    }

    match fs::write(output_path, output_content) {
        Ok(_) => {
            println!("chord was {:.4}, {} points normalized -> saved to {}", chord, points.len(), output_path);
        }
        Err(err) => {
            eprintln!("Error writing to output file '{}': {}", output_path, err);
            process::exit(1);
        }
    }
}
