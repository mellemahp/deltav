#[macro_use]
extern crate structopt;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use structopt::StructOpt;
use std::f32;
use std::f32::consts::PI;
use std::fs::File;
use std::io::prelude::*;
use std::process;
use std::path::PathBuf;

#[derive(StructOpt, Debug)]
#[structopt(name = "Delta V Calculator",
            about = "A simple co-planar hohmann transfer calculator")]
struct Opt {
    #[structopt(short = "d",
                long = "debug",
                help = "Activate debug mode")]
    debug: bool,

    #[structopt(long = "in",
                help = "Input file (TOML format)",
                default_value = "config.toml",
                parse(from_os_str))]
    input: PathBuf,

    #[structopt(long = "out",
                help = "Output file, stdout if not present",
                parse(from_os_str))]
    output: Option<PathBuf>,

}

#[derive(Deserialize, Debug)]
pub struct Config {
    orbit1: OrbitConfig,
    orbit2: OrbitConfig,
}

#[derive(Deserialize, Debug)]
struct OrbitConfig {
    peri: Option<f32>,
    apo: Option<f32>,
    ecc: Option<f32>,
    vel_p: Option<f32>,
    vel_a: Option<f32>,
    spk_id: Option<i32>,
}

impl Config {
    /// Attempt to load and parse the config file into our Config struct.
    /// If a file cannot be found, return an error to the user.
    /// If we find a file but cannot parse it, panic
    pub fn parse(path: PathBuf) -> Config {
        let mut config_toml = String::new();

        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(_)  => {
                panic!("Could not find config file! Please enter valid path.");
            }
        };

        file.read_to_string(&mut config_toml)
            .unwrap_or_else(|err| panic!("Error while reading config: [{}]",
                                         err));
        let config: Config =  match toml::from_str(&config_toml) {
                Ok(table) => table,
                Err(_) => {
                    println!("failed to parse TOML configuration");
                    process::exit(2);
                }
        };

        return config
    }
}


#[derive(Debug)]
enum OrbError {
    IncompatibleArgs,
    TooFewArgs,
}

#[derive(Debug)]
struct Orbit {
    ra: f32,
    rp: f32,
    mu: f32,
    h: f32,
    ecc: f32,
}

impl Orbit {
    fn from_config(config: OrbitConfig) -> Orbit {
        let mu = find_mu(&config.spk_id);
        let orbit = match config {
            OrbitConfig { peri: Some(rp), apo: Some(ra), ..} => {
                Ok(Orbit{
                    ra: rp,
                    rp: ra,
                    mu: mu,
                    h: (2.0 * mu).sqrt() * ((rp * ra) / (rp + ra)).sqrt(),
                    ecc: (ra - rp) / (ra + rp),
                })
            },
            OrbitConfig { peri: Some(rp), ..} => {
                match config {
                    OrbitConfig { ecc: Some(ecc), ..} => {
                        Ok(Orbit{
                            ra: (rp * (ecc + 1.0) / (rp - 1.0 )).abs(),
                            rp: rp,
                            mu: mu,
                            h: (rp * mu * (1.0 + ecc)).sqrt(),
                            ecc: ecc,
                        })
                    },
                    OrbitConfig { vel_p: Some(vp), ..} => {
                        Ok(Orbit{
                            ra: (rp * vp).powi(2) / (mu * (1.0 - (rp * vp).powi(2) /
                                                           (rp * mu ) - 1.0)),
                            rp: rp,
                            mu: mu,
                            h: rp * vp,
                            ecc: (rp * vp).powi(2) / (rp * mu ) - 1.0,
                        })
                    },
                    OrbitConfig { vel_a: Some(_), .. }=> {
                        Err(OrbError::IncompatibleArgs)
                    },
                    _ => Err(OrbError::TooFewArgs),
                }
            },
            OrbitConfig { apo: Some(ra), ..} => {
                match config {
                    OrbitConfig{ ecc: Some(ecc), ..} =>{
                        Ok(Orbit{
                            ra: ra,
                            rp: ra * (1.0 - ecc) / (1.0 + ecc),
                            mu: mu,
                            h: (mu * ra  * (1.0 - ecc)).sqrt(),
                            ecc: ecc,
                        })
                    },
                    OrbitConfig{vel_a: Some(va), ..} =>{
                        Ok(Orbit{
                            ra: ra,
                            rp: (ra  * va).powi(2) / (ra * (va).powi(2) - 2.0 * mu),
                            mu: mu,
                            h: ra * va,
                            ecc: 1.0 - (ra * (va).powi(2) ) / (ra * mu),
                        })
                    },
                    OrbitConfig{vel_p: Some(_), ..} =>{
                        Err(OrbError::IncompatibleArgs)
                    }
                    _=> Err(OrbError::TooFewArgs),
                }
            },
            _ => Err(OrbError::TooFewArgs),
        };
        return orbit.unwrap()
    }

    fn orb_period(&self) -> f32 {
        let period = 2.0 * PI / self.mu.sqrt() * ((self.rp + self.ra) /
                                                  2.0).powf(3.0 / 2.0);
        return period

    }
}

fn find_mu(spk_id: &Option<i32> ) -> f32 {
    let mu = match spk_id.expect("Central Body settings are borked!") {
        // All values in (km^3)(s^-2)
        0 => 1.327 * 10_f32.powi(11), // Sun
        1 => 2.203 * 10_f32.powi(4), // Mercury
        2 => 3.257 * 10_f32.powi(5), // Venus
        3 => 3.986 * 10_f32.powi(5), // Earth
        301 => 4902.799 , // Moon
        4 => 4.305 * 10_f32.powi(4), // Mars
        5 => 1.268 * 10_f32.powi(8), // Jupiter
        6 => 3.794 * 10_f32.powi(7), // Saturn
        7 => 5.794 * 10_f32.powi(6), // Uranus
        8 => 6.809 * 10_f32.powi(6), // Neptune
        _ => panic!("Only SPKIDs 0-8 (and 301) are currently supported!"),
    };
    mu
}

fn find_transfer_ellipse(orbit1: &Orbit, orbit2: &Orbit) -> Orbit {
    let config = OrbitConfig{
        peri: Some(orbit1.rp),
        apo: Some(orbit2.ra),
        spk_id: Some(3),
        ecc: None, vel_p: None, vel_a: None,
    };

    let transfer = Orbit::from_config(config);
    return transfer
}

fn process_inputs(opt: Opt) {
    let config = Config::parse(opt.input);
    let orbit1 = Orbit::from_config(config.orbit1);
    let orbit2 = Orbit::from_config(config.orbit2);
    let transfer = find_transfer_ellipse(&orbit1, &orbit2);
    println!("Orbit1: {:?}", orbit1);
    println!("Orbit2: {:?}", orbit2);
    println!("Transfer: {:?}", transfer);
}

fn main() {
    let opt = Opt::from_args();
    process_inputs(opt);
}
