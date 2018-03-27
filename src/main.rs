#[macro_use]
extern crate structopt;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(name = "periapsis",
                short = "p1",

                // DEBUG START
                default_value = "3.9",
                // DEBUG END

                help ="Periapsis of the initial Orbit (default units km)")]
    peri: Option<f32>,
    #[structopt(name = "apoapsis",
                long = "a1",
                help = "Apoapsis of the initial Orbit (default units km)")]
    apo: Option<f32>,
    #[structopt(name = "eccentricity",
                long = "e",

                // DEBUG START
                default_value = "0.0",
                // DEBUG END

                help ="Eccentricty of the initial Orbit (default units km)")]
    ecc: Option<f32>,
    #[structopt(name = "periapsis velocity",
                long = "vp1",
                help ="Velocity of orbit at periapsis (default units km/s)")]
    vel_peri1: Option<f32>,
    #[structopt(name = "apoapsis velocity",
                long = "va1",
                help ="Periapsis of orbit at apoapsis (default units km/s)")]
    vel_apo1: Option<f32>,


    #[structopt(name = "central body",
                long = "cb",
                default_value = "3",
                help ="central body orbit is defined around (horizons ID)")]
    spk_id: i32,
}

#[derive(Debug)]
struct Orbit {
    apoapsis: f32,
    periapsis: f32,
}


fn angular_momentum(mu: f32, rp: f32, ra: f32) -> f32 {
    let h = (2.0 * mu).sqrt() * ((rp * ra) / (rp + ra)).sqrt();
    h
}


fn args_to_orbs(opt: Opt) -> Orbit {
    match opt {
        Opt{ peri: Some(_), apo: Some(_), ..} => {
            let orbit1 = Orbit{
                apoapsis: opt.apo.unwrap(),
                periapsis: opt.peri.unwrap(),
            };
            return orbit1;
        },
        Opt {peri: Some(_), ecc: Some(_), ..} => {
            let orbit1 = Orbit{
                apoapsis: (opt.peri.unwrap() * (opt.ecc.unwrap() + 1.0) /
                           (opt.ecc.unwrap() -1.0 )).abs(),
                periapsis: opt.peri.unwrap()
            };
            return orbit1;
        },
        Opt {peri: Some(_), vel_peri1: Some(_), .. } => {
            let orbit1 = Orbit{
                // PLACEHOLDER VALUE START
                apoapsis: 3.5,
                // PLACEHOLDER VALUE END
                periapsis: opt.peri.unwrap(),
            };
            return orbit1;
        },
        Opt {apo: Some(_), ecc: Some(_), ..} => {
            let orbit1 = Orbit{
                apoapsis: opt.apo.unwrap(),
                periapsis: (opt.apo.unwrap() * ( 1.0 - opt.ecc.unwrap()) /
                            (opt.ecc.unwrap() + 1.0)).abs()
            };
            return orbit1;
        },
        Opt {apo: Some(_), vel_apo1: Some(_), .. } => {
            let orbit1 = Orbit{
                // PLACEHOLDER VALUE START
                apoapsis: opt.apo.unwrap(),
                // PLACEHOLDER VALUE END
                periapsis: 3.5,
            };
            return orbit1;
        },

       Opt {apo: Some(_), vel_peri1: Some(_), .. } |
       Opt {ecc: Some(_), vel_peri1: Some(_), .. } |
       Opt {ecc: Some(_), vel_apo1: Some(_), .. } |
       Opt {peri: Some(_), vel_apo1: Some(_), .. } => {
            panic!("The arguments you entered are incompatible!");
       }
        _ => panic!("Not Enough Arguments! (need at least 2!)")
    }
}


fn main() {
    let opt = Opt::from_args();
    println!("{:?}",
             opt);
   //  let h = angular_momentum(2.0,1.0,2.0);
    let orbit1 = args_to_orbs(opt);
    println!("Orbit1: {:?}", orbit1);

}
