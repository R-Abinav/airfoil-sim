mod airfoil;
mod parser;
mod panel_method;
mod results;

use std::path::Path;

fn main() {
    let data_dir   = Path::new("data/airfoils");
    let output_dir = Path::new("output");

    //alpha sweep config
    let alpha_start =  -5.0_f64;
    let alpha_end   =  15.0_f64;
    let alpha_step  =   1.0_f64;
    let v_inf       =   1.0_f64;

    let alphas: Vec<f64> = {
        let mut a = alpha_start;
        let mut v = Vec::new();
        while a <= alpha_end {
            v.push(a);
            a += alpha_step;
        }
        v
    };

    //load all airfoils from data/airfoils/ 
    let airfoils = parser::load_all(data_dir);

    if airfoils.is_empty() {
        eprintln!("No .dat files found in {:?}. Exiting.", data_dir);
        return;
    }

    //run simulation for each airfoil 
    for airfoil in &airfoils {
        println!("\nSimulating: {}", airfoil.name);
        println!("  Panels: {}", airfoil.points.len() - 1);

        let mut cls = Vec::new();

        for &alpha in &alphas {
            let results = panel_method::solve(airfoil, alpha, v_inf);

            //write Cp distribution for this alpha
            if let Err(e) = results::write_cp(&results, &airfoil.name, alpha, output_dir) {
                eprintln!("  Error writing Cp: {}", e);
            }

            cls.push(results.cl);

            println!("  alpha = {:+5.1}°  Cl = {:.4}", alpha, results.cl);
        }

        //write Cl vs alpha sweep
        if let Err(e) = results::write_cl(&airfoil.name, &alphas, &cls, output_dir) {
            eprintln!("  Error writing Cl: {}", e);
        }
    }

    println!("\nDone. Results in {:?}", output_dir);
}