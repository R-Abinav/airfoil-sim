//just defines what an airfoil is in our program 
//if needed - generates NACA 4 digit coordinates (as fallback - if no .dat files present)

//single 2D point on the airfoil surface
#[derive(Clone, Debug)]
pub struct Point{
    pub x: f64, 
    pub y: f64,
}

//holds the full airfoil
//name and ordered surface coordinates
//coordinates must go from:
//trailing edge -> upper surface -> leading edge -> lower surface -> trailing edge (standard selig format)
#[derive(Clone, Debug)]
pub struct Airfoil{
    pub name: String,
    pub points: Vec<Point>,
}

impl Airfoil{
    //generate a NACA 4-digit airfoil programmatically
    //e.g. NACA 2412 -> m=0.02, p=0.4, t=0.12
    pub fn naca4(code: &str, n_points: usize) -> Self{
        let digits: Vec<u32> = code.chars()
                                   .map(|c| c.to_digit(10).expect("invalid NACA code"))
                                   .collect();

        assert!(digits.len() == 4, "Must be a 4-digit NACA code");

        let m = digits[0] as f64 / 100.0; //max camber
        let p = digits[0] as f64 / 10.0; //location of max camber
        let t = (digits[2] * 10 + digits[3]) as f64 / 100.0; //thickness

        let mut points = Vec::new();

        //cosine spacing for denser points near landing/trailing edge
        for i in 0..n_points{
            let beta = std::f64::consts::PI * i as f64 / n_points as f64;
            let x = 0.5 * (1.0 - beta.cos());

            //thickness distribution (NACA formula)
            let yt = 5.0 * t * (
                0.2969 * x.sqrt()
                - 0.1260 * x
                - 0.3516 * x.powi(2)
                + 0.2843 * x.powi(3)
                - 0.1015 * x.powi(4)
            );

            //camber line and slope
            let (yc, dyc_dx) = if p == 0.0 || m == 0.0{
                (0.0, 0.0)
            }else if x < p{
                (
                    m / p.powi(2) * (2.0 * p * x - x.powi(2)),
                    2.0 * m / p.powi(2) * (p - x),
                )
            }else{
                (
                    m / (1.0 - p).powi(2) * (1.0 - 2.0 * p + 2.0 * p * x - x.powi(2)),
                    2.0 * m / (1.0 - p).powi(2) * (p - x),
                )
            };

            let theta = dyc_dx.atan();

            //upper and lower surface points
            points.push(Point {
                x: x - yt * theta.sin(),
                y: yc + yt * theta.cos(),
            });

            if i != 0 && i != n_points {
                points.push(Point {
                    x: x + yt * theta.sin(),
                    y: yc - yt * theta.cos(),
                });
            }
        }

        Airfoil {
            name: format!("NACA {}", code),
            points,
        }
    }
}
