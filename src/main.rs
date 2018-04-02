#[macro_use]
extern crate structopt;
#[macro_use]
extern crate serde_derive;
extern crate toml;
#[allow(unused_imports)]
use structopt::StructOpt;
use std::path::PathBuf;
use std::f32;
use std::f32::consts::PI;
use std::fs::File;
use std::io::prelude::*;
use std::process;

#[derive(StructOpt, Debug)]
#[structopt(name = "Delta V Calculator",
            author = "Hunter Mellema",
            about = "A simple command line tool to calculate Hohmann transfers")]
struct Opt {
    #[structopt(long = "d",
                help = "Activate debug mode")]
    debug: bool,

    #[structopt(long = "in",
                help = "Input file (TOML format)",
                parse(from_os_str))]
    input: PathBuf,

    #[structopt(long = "out",
                help = "Output file, stdout if not present",
                parse(from_os_str))]
    output: Option<PathBuf>,

    #[structopt(long = "cb",
                default_value = "3",
                help ="CB system is defined around (horizons ID)")]
    spk_id: i32,

    #[structopt(subcommand)]
    unit_sys: Units,
}

#[derive(StructOpt, Debug)]
enum Units {
    #[structopt(name = "km")]
    Kilometers,
    #[structopt(name = "m")]
    Meters,
    #[structopt(name = "au")]
    AstroUnit,
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
    pub fn parse(path: String) -> Config {
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
                Err(err) => {
                    println!("failed to parse TOML configuration '{}': {}", path,
                             err);
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
            OrbitConfig { peri: Some(_), ..} => {
                match config {
                    OrbitConfig { apo: Some(_), .. } => {
                        Ok(Orbit{
                            ra: config.apo.unwrap(),
                            rp: config.peri.unwrap(),
                            mu: mu,
                            h: rp_ra_to_h(&config.apo, &config.peri, &mu),
                            ecc: (config.apo.unwrap() - config.peri.unwrap()) /
                                (config.apo.unwrap() + config.peri.unwrap()),
                        })
                    },
                    OrbitConfig { ecc: Some(_), ..} => {
                        Ok(Orbit{
                            ra: (config.peri.unwrap() * (config.ecc.unwrap() + 1.0) /
                                 (config.ecc.unwrap() -1.0 )).abs(),
                            rp: config.peri.unwrap(),
                            mu: mu,
                            h: (config.peri.unwrap() * mu * (1.0 + config.ecc.unwrap())).sqrt(),
                            ecc: config.ecc.unwrap(),
                        })
                    },
                    OrbitConfig { vel_p: Some(_), ..} => {
                        Ok(Orbit{
                            ra: (config.peri.unwrap() *
                                 config.vel_p.unwrap()).powi(2) /
                                (mu * (1.0 - (config.peri.unwrap() *
                                            config.vel_p.unwrap()).powi(2) /
                                     (config.peri.unwrap() * mu ) - 1.0)),
                            rp: config.peri.unwrap(),
                            mu: mu,
                            h: config.peri.unwrap() * config.vel_p.unwrap(),
                            ecc: (config.peri.unwrap() * config.vel_p.unwrap()).powi(2) /
                                (config.peri.unwrap() * mu ) - 1.0,
                        })
                    },
                    OrbitConfig { vel_a: Some(_), .. }=> {
                        Err(OrbError::IncompatibleArgs)
                    },
                    _ => Err(OrbError::TooFewArgs),
                }
            },
            OrbitConfig { apo: Some(_), ..} => {
                match config {
                    OrbitConfig{ecc: Some(_), ..} =>{
                        Ok(Orbit{
                            ra: config.peri.unwrap(),
                            rp: (config.apo.unwrap() * ( 1.0 - config.ecc.unwrap()) /
                                 (config.ecc.unwrap() + 1.0)).abs(),
                            mu: mu,
                            h: (mu * config.apo.unwrap() * (1.0 - config.ecc.unwrap())).sqrt(),
                            ecc: config.ecc.unwrap(),
                        })
                    },
                    OrbitConfig{vel_a: Some(_), ..} =>{
                        Ok(Orbit{
                            ra: config.apo.unwrap(),
                            rp: (config.apo.unwrap() * config.vel_a.unwrap()).powi(2) /
                                (config.apo.unwrap() * (config.vel_a.unwrap()).powi(2) -
                                2.0 * mu),
                            mu: mu,
                            h: config.apo.unwrap() * config.vel_a.unwrap(),
                            ecc: 1.0 - (config.apo.unwrap() * config.vel_a.unwrap()).powi(2) /
                                (config.apo.unwrap() * mu),
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


fn rp_ra_to_h(peri: &Option<f32>, apo: &Option<f32>, mu: &f32) -> f32 {
    let rp = peri.expect("Periapsis altitude must not be \"None\"!");
    let ra = apo.expect("Apoapsis altitude must not be \"None\"!");
    let h = (2.0 * mu).sqrt() * ((rp * ra) / (rp + ra)).sqrt();
    return h
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

/*
// Attempt to load and parse the config file into our Config struct.
/// If a file cannot be found, return a default Config.
/// If we find a file but cannot parse it, panic
/*pub fn parse_toml(Pathbuf) -> Config {
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(_)  => {
            error!("Could not find config file, using default!");
            return Config::new();
        }
    };

    file.read_to_string(&mut config_toml)
            .unwrap_or_else(|err| panic!("Error while reading config: [{}]", err));

    let mut parser = Parser::new(&config_toml);
    let toml = parser.parse();

    if toml.is_none() {
        for err in &parser.errors {
            let (loline, locol) = parser.to_linecol(err.lo);
            let (hiline, hicol) = parser.to_linecol(err.hi);
            println!("{}:{}:{}-{}:{} error: {}",
                     path, loline, locol, hiline, hicol, err.desc);
        }
        panic!("Exiting server");
    }

    let config = Value::Table(toml.unwrap());
    match toml::decode(config) {
        Some(t) => t,
        None => panic!("Error while deserializing config")
    }
}
*/*/
/*
fn process_inputs(mu: f32) -> (Orbit, Orbit) {
    let orbit1 = Orbit { rp: 1507.0, ra: 1507.0, ecc: 3.1,  mu: mu, h: 0.0};
    // Molniya orbit
    let orbit2 = Orbit { rp: 1507.0, ra: 39305.0,ecc: 2.4,  mu: mu, h: 0.0};
    return (orbit1, orbit2)
}
*/

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


fn main() {
    // let opt = Opt::from_args();

    let example = OrbitConfig {
        peri: Some(1507.0),
        apo: Some(1507.0),
        ecc: None,
        vel_p: None,
        vel_a: None,
        spk_id: Some(3),
    };
    let example2 = OrbitConfig {
        peri: Some(2000.0),
        apo: Some(5000.0),
        ecc: None,
        vel_p: None,
        vel_a: None,
        spk_id: Some(3),
    };

    let orbit1 = Orbit::from_config(example);
    let orbit2 = Orbit::from_config(example2);
    let period1 = orbit1.orb_period();
    let period2 = orbit2.orb_period();
    let transfer = find_transfer_ellipse(&orbit1, &orbit2);
    println!("Orbit1: {:?}", orbit1);
    println!("Orbit2: {:?}", orbit2);
    println!("TransferOrbit: {:?}", transfer);
    let filename = String::from("config.toml");
    let config = Config::parse(filename);
    println!("orbit1!: {:?}", config.orbit1);
    println!("orbit2!: {:?}", config.orbit2);
}
