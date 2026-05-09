//what all this should do:
//1. Build panels from airfoil points
//2. Build the influence matrix
//3. RHS Vector
//4. Kutta condition
//5. Solve (Use something like LU Decomposition from the crate nalgebra)
//6. compute results

use std::f64::consts::PI;
use nalgebra::{DMatrix, DVector};
use crate::airfoil::{Airfoil, Point};

//represents a single panel between two airfoil surface points
#[derive(Clone, Debug)]
pub struct Panel{
    pub x1: f64, pub y1: f64,   //start point
    pub x2: f64, pub y2: f64,   //end point
    pub xm: f64, pub ym: f64,   //midpoint (control point)
    pub length: f64,             //panel length delta(s)
    pub phi: f64,                //panel angle from x-axis
    pub beta: f64,               //normal angle (phi + pi/2)
}

//full results for one airfoil at one angle of attack 
#[derive(Debug)]
pub struct PanelResults{
    pub cp: Vec<f64>, //pressure coeff at each panel
    pub cl: f64,    //lift coeff
    pub panels: Vec<Panel> //panels for x/c output
}

//build panels from airfoil coordinates
pub fn build_panels(airfoil: &Airfoil) -> Vec<Panel>{
    let pts = &airfoil.points;
    let n = pts.len() - 1; //num of panels = num_points - 1

    (0..n).map(|i| {
        let (x1, y1) = (pts[i].x,     pts[i].y);
        let (x2, y2) = (pts[i+1].x,   pts[i+1].y);

        let xm     = 0.5 * (x1 + x2);
        let ym     = 0.5 * (y1 + y2);
        let length = ((x2-x1).powi(2) + (y2-y1).powi(2)).sqrt();
        let phi    = (y2 - y1).atan2(x2 - x1);
        let beta   = phi + PI / 2.0;

        Panel { x1, y1, x2, y2, xm, ym, length, phi, beta }
    }).collect()
}

//compute normal and tangential influence coeff
//of panel "j" on control point "i"
fn influence(panels: &[Panel], i: usize, j: usize) -> (f64, f64) {
    //self influence - known analytical result
    if i == j {
        return (0.5, 0.0);
    }

    let pi = &panels[i];
    let pj = &panels[j];

    //transform control point i into panel j's local frame
    let dx = pi.xm - pj.x1;
    let dy = pi.ym - pj.y1;

    let cos_j = pj.phi.cos();
    let sin_j = pj.phi.sin();

    //local coordinates
    let x =  dx * cos_j + dy * sin_j;
    let y = -dx * sin_j + dy * cos_j;

    let sj = pj.length;

    //distances to panel endpoints
    let r1 = (x.powi(2) + y.powi(2)).sqrt();
    let r2 = ((x - sj).powi(2) + y.powi(2)).sqrt();

    //angles to panel endpoints
    let b1 = y.atan2(x);
    let b2 = y.atan2(x - sj);
    let db = b2 - b1;

    let angle_diff = pi.phi - pj.phi;
    let cos_d = angle_diff.cos();
    let sin_d = angle_diff.sin();

    let ln_r = (r1 / r2).ln();

    //normal influence coefficient
    let a = (sin_d * ln_r + cos_d * db) / (2.0 * PI);

    //tangential influence coefficient
    let b = (cos_d * ln_r - sin_d * db) / (2.0 * PI);

    (a, b)
}

//run the full vortex panel method
//alpha_deg: angle of attack in degrees
//v_inf: freestream velocity (usually 1.0)
pub fn solve(airfoil: &Airfoil, alpha_deg: f64, v_inf: f64) -> PanelResults {
    let panels = build_panels(airfoil);
    let n = panels.len();
    let alpha = alpha_deg.to_radians();

    //build influence matrix A and RHS vector b 
    let mut a_mat = DMatrix::<f64>::zeros(n, n);
    let mut b_tan = DMatrix::<f64>::zeros(n, n); //tangential coeffs for Cp
    let mut rhs   = DVector::<f64>::zeros(n);

    for i in 0..n {
        for j in 0..n {
            let (a_ij, b_ij) = influence(&panels, i, j);
            a_mat[(i, j)] = a_ij;
            b_tan[(i, j)] = b_ij;
        }

        //RHS: freestream normal component at panel i
        rhs[i] = v_inf * (panels[i].phi - alpha).sin();
    }

    //apply Kutta condition
    //replace last row
    for j in 0..n {
        a_mat[(n-1, j)] = 0.0;
    }
    a_mat[(n-1, 0)]   = 1.0;
    a_mat[(n-1, n-1)] = 1.0;
    rhs[n-1]          = 0.0;

    //solve Agamma = b using LU decomposition 
    let gamma = a_mat
        .lu()
        .solve(&rhs)
        .expect("Linear solve failed");

    //compute tangential velocity and Cp at each panel
    let mut cp = vec![0.0; n];

    for i in 0..n {
        //freestream tangential component
        let vt_inf = v_inf * (panels[i].phi - alpha).cos();

        //induced tangential velocity from all panels
        let vt_induced: f64 = (0..n)
            .map(|j| b_tan[(i, j)] * gamma[j])
            .sum();

        let vt = vt_inf + vt_induced;
        cp[i] = 1.0 - (vt / v_inf).powi(2);
    }

    //lift coefficient via Kutta
    let cl = 2.0 / v_inf
        * (0..n)
            .map(|j| gamma[j] * panels[j].length)
            .sum::<f64>();

    PanelResults { cp, cl, panels }
}