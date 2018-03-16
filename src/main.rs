#[macro_use]
extern crate structopt;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(name = "periapsis",
                short = "p1",
                help ="Periapsis of the initial Orbit (default units km)")]
    peri: Option<f32>,
    #[structopt(name = "apoapsis",
                long = "a1",
                help = "Apoapsis of the initial Orbit (default units km)")]
    apo: Option<f32>,
    #[structopt(name = "eccentricity",
                long = "e",
                default_value = "0",
                help ="Eccentricty of the initial Orbit (default units km)")]
    ecc: f32,
    #[structopt(name = "periapsis velocity",
                long = "vp",
                help ="Velocity of orbit at periapsis (default units km/s)")]
    vel_peri: Option<f32>,
    #[structopt(name = "apoapsis velocity",
                long = "va",
                help ="Periapsis of orbit at apoapsis (default units km/s)")]
    vel_apo: Option<f32>,
    #[structopt(name = "central body",
                long = "cb",
                default_value = "3",
                help ="central body orbit is defined around (horizons ID)")]
    spk_id: i32,
    // #[structopt(subcommand)]
    // dist_unit: Option<Units>,

}
/*
#[derive(StructOpt)]
enum Units {
    #[structopt(short = "km")]
    kilometers,
    #[structopt(short = "m")]
    meters,
    #[structopt(short = "mi")]
    miles,
    #[structopt(short = "au")]
    astro_unit,
}

#[derive(StructOpt)]
enum VelocityUnit {
    #[structopt(short = "m/s")]
    m_sec,
    #[structopt(short = "ft/s")]
    ft_sec,
    #[structopt(short = "km/s")]
    km_sec,
    #[structopt(short = "mi/s")]
    mi_sec,
    #[structopt(short = "km/hr")]
    km_hr,
    #[structopt(short = "mi/hr")]
    mi_hr,
}
 */

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
