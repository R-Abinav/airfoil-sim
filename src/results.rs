use std::fs;
use std::path::Path;
use csv::Writer;
use serde::Serialize;
use crate::panel_method::PanelResults;

//one row in the Cp CSV
#[derive(Serialize)]
struct CpRow {
    x_over_c: f64,
    cp: f64,
}

//one row in the Cl CSV
#[derive(Serialize)]
struct ClRow {
    alpha_deg: f64,
    cl: f64,
}

//write Cp distribution to output/{name}_alpha{alpha}_cp.csv
pub fn write_cp(
    results: &PanelResults,
    name: &str,
    alpha_deg: f64,
    out_dir: &Path,
) -> Result<(), String> {
    fs::create_dir_all(out_dir)
        .map_err(|e| format!("Could not create output dir: {}", e))?;

    //sanitize name for filename — replace spaces with underscores
    let safe_name = name.replace(' ', "_");
    let alpha_int = alpha_deg as i32;
    let filename  = format!("{}_alpha{}_cp.csv", safe_name, alpha_int);
    let path      = out_dir.join(filename);

    let mut wtr = Writer::from_path(&path)
        .map_err(|e| format!("Could not create CSV {:?}: {}", path, e))?;

    for (panel, &cp) in results.panels.iter().zip(results.cp.iter()) {
        wtr.serialize(CpRow {
            x_over_c: panel.xm,
            cp,
        }).map_err(|e| format!("CSV write error: {}", e))?;
    }

    wtr.flush()
        .map_err(|e| format!("CSV flush error: {}", e))?;

    println!("  Written: {:?}", path);
    Ok(())
}

//write Cl vs alpha sweep to output/{name}_cl.csv
pub fn write_cl(
    name: &str,
    alphas: &[f64],
    cls: &[f64],
    out_dir: &Path,
) -> Result<(), String> {
    fs::create_dir_all(out_dir)
        .map_err(|e| format!("Could not create output dir: {}", e))?;

    let safe_name = name.replace(' ', "_");
    let filename  = format!("{}_cl.csv", safe_name);
    let path      = out_dir.join(filename);

    let mut wtr = Writer::from_path(&path)
        .map_err(|e| format!("Could not create CSV {:?}: {}", path, e))?;

    for (&alpha, &cl) in alphas.iter().zip(cls.iter()) {
        wtr.serialize(ClRow {
            alpha_deg: alpha,
            cl,
        }).map_err(|e| format!("CSV write error: {}", e))?;
    }

    wtr.flush()
        .map_err(|e| format!("CSV flush error: {}", e))?;

    println!("  Written: {:?}", path);
    Ok(())
}