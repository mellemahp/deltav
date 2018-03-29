#[macro_use]
extern crate structopt;

use structopt::StructOpt;
use std::path::PathBuf;
use std::f32;
use std::f32::consts::PI;

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

#[derive(Debug)]
struct OrbitConfig {
    peri: Option<f32>,
    apo: Option<f32>,
    ecc: Option<f32>,
    vel_peri1: Option<f32>,
    vel_apo1: Option<f32>,
    spk_id: Option<i32>,
}

#[derive(Debug)]
enum OrbError {
    IncompatibleArgs,
    TooFewArgs,
    Borked,
}

pub type OrbResult = Result< Orbit, OrbError>;

#[derive(Debug)]
struct Orbit {
    ra: f32,
    rp: f32,
    mu: f32,
    h: Option<f32>,
}

impl Orbit {
    fn from_config(config: OrbitConfig) -> Orbit {
        let mu = find_mu(&config.spk_id);
        let mut orbit = match config {
            OrbitConfig { peri: Some(_), apo: Some(_), ..} => {
                Ok(Orbit{
                    ra: config.apo.unwrap(),
                    rp: config.peri.unwrap(),
                    mu: mu,
                    h: None,
                })
            },
            OrbitConfig {peri: Some(_), ecc: Some(_), ..} => {
                Ok(Orbit{
                    ra: (config.peri.unwrap() * (config.ecc.unwrap() + 1.0) /
                               (config.ecc.unwrap() -1.0 )).abs(),
                    rp: config.peri.unwrap(),
                    mu: mu,
                    h: None,
                })
            },
            OrbitConfig {peri: Some(_), vel_peri1: Some(_), .. } => {
                Ok(Orbit{
                    // PLACEHOLDER VALUE START
                    ra: 3.5,
                    // PLACEHOLDER VALUE END
                    rp: config.peri.unwrap(),
                    mu: mu,
                    h: None,
                })
            },
            OrbitConfig {apo: Some(_), ecc: Some(_), ..} => {
                Ok(Orbit{
                    ra: config.apo.unwrap(),
                    rp: (config.apo.unwrap() * ( 1.0 - config.ecc.unwrap()) /
                                (config.ecc.unwrap() + 1.0)).abs(),
                    mu: mu,
                    h: None,
                })
            },
            OrbitConfig {apo: Some(_), vel_apo1: Some(_), .. } => {
                Ok(Orbit{
                    ra: config.apo.unwrap(),
                    // PLACEHOLDER VALUE START
                    rp: 3.5,
                    // PLACEHOLDER VALUE END
                    mu: mu,
                    h: None,
                })
            },
            OrbitConfig {apo: Some(_), vel_peri1: Some(_), .. } |
            OrbitConfig {ecc: Some(_), vel_peri1: Some(_), .. } |
            OrbitConfig {ecc: Some(_), vel_apo1: Some(_), .. } |
            OrbitConfig {peri: Some(_), vel_apo1: Some(_), .. } => {
                Err(OrbError::IncompatibleArgs)
            },
            _ => {
                Err(OrbError::TooFewArgs)
            },
        };
        return orbit.unwrap()
    }

    fn angular_momentum(&mut self) {
        self.h = Some((2.0 * self.mu).sqrt() * ((self.rp * self.ra) /
                                               (self.rp + self.ra)).sqrt());
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

fn process_inputs(mu: f32) -> (Orbit, Orbit) {
    let orbit1 = Orbit { rp: 1507.0, ra: 1507.0, mu: mu, h: Some(0.0)};
    // Molniya orbit
    let orbit2 = Orbit { rp: 1507.0, ra: 39305.0, mu: mu, h: Some(0.0)};
    return (orbit1, orbit2)
}



fn main() {
    // let opt = Opt::from_args();
    let x = Some(4);
    let mu = find_mu(&x);
    let orbs = process_inputs(mu);
    println!("Orbit1: {:?}", orbs.0);
    println!("Orbit2: {:?}", orbs.1);

    let example = OrbitConfig {
        peri: Some(1507.0),
        apo: Some(39305.0),
        ecc: None,
        vel_peri1: None,
        vel_apo1: None,
        spk_id: Some(3),
    };

    let mut orbit3 = Orbit::from_config(example);
    orbit3.angular_momentum();
    let period = orbit3.orb_period();
    println!("Orbit3: {:?}", orbit3);
    println!("Period: {:?}", period)
}
