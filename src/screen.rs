// screen

use std::f64;
use rand::Rng;

use super::ray::*;
use super::ray::algebra::*;
use super::ray::geometry::*;
use super::ray::optics::*;


pub struct Rgb(i32, i32, i32);

pub struct Screen {
  pub nphoton: i32,
  pub progressive: bool,
  pub xreso: i32,
  pub yreso: i32,
  pub antialias: bool,
  pub n_sample_photon: i32,
  pub use_classic_for_direct: bool,
  pub radius: Flt,
  pub pfilter: PhotonFilter,
  pub ambient: Radiance,
  pub max_radiance: Flt,
  pub eye_pos: Position3,
  pub eye_dir: Direction3,
  pub focus: Flt,
  pub screen_map: Vec<(Flt, Flt)>,
  pub pnm_header: Vec<String>,
  pub radiance_to_rgb: fn(Radiance) -> Rgb,
  pub generate_ray: fn(Flt, Flt) -> Ray,
}

pub const GAMMA: Flt  = 1.0 / 2.2;
pub const RGBMAX: Flt = 255.0;

//const DEFCONF: HashMap = HashMap

pub fn read_screen(file: &str) -> Screen {
  let target = Vector3::new_pos(0.0, 2.0, 0.0);
  let eyepos = Vector3::new_pos(0.0, 2.0, -4.5);
  let focallen: Flt = 50.0 / 1000.0;
  let fnumber: Flt = 5.0;

  let scr = Screen {
    nphoton: 10000,
    progressive: true,
    xreso: 256,
    yreso: 256,
    antialias: true,
    n_sample_photon: 500,
    use_classic_for_direct: true,
    radius: 0.2,
    pfilter: PhotonFilter::Gauss,
    ambient: Radiance(0.001, 0.001, 0.001),
    max_radiance: 0.01,
    eye_pos: eyepos,
    eye_dir: (target - eyepos).normalize().unwrap(),
    focus: 7.0,
    screen_map: Vec<(Flt, Flt)>,
    pnm_header: Vec<String>,
    radiance_to_rgb: fn(Radiance) -> Rgb,
    generate_ray: fn(Flt, Flt) -> Ray,
  };
  scr
}

pub fn rgb_to_string(c: &Rgb) -> String {
  format!("{} {} {}", c.0, c.1, c.2)
}

pub fn radiance_to_string(r: &Radiance) -> String {
  format!("{} {} {}", r.0, r.1, r.2)
}

pub fn rgb_to_radiance(scr: &Screen, c: &Rgb) -> Radiance {
  let mag = scr.max_radiance / RGBMAX;
  Radiance(c.0 as f64 * mag, c.1 as f64 * mag, c.2 as f64 * mag)
}

//--------------------
// private functions

fn make_generate_ray(aa_flag: &bool, prog_flag: &bool, epos: &Position3,
  target: &Direction3, xr: &i32, yr: &i32, udir: &Direction3, fd: Flt, fl: Flt)
  -> fn(Flt, Flt) -> Ray {

}

fn generate_ray0(aa_flag: &bool, prog_flag: &bool, blur_flag: &bool,
  e: &Position3, o: &Position3, esx: &Direction3, esy: &Direction3,
  eex: &Direction3, eey: &Direction3, (y, x): &(Flt, Flt))
  -> Ray {
  let mut rng = rand::thread_rng();
  let blur = if *blur_flag == true {
    let r1: Flt = rng.gen_range(-0.5, 0.5);
    let r2: Flt = rng.gen_range(-0.5, 0.5);
    r1 * *eex + r2 * *eey
  } else {
    Vector3::O
  };
  let (r3, r4) = if *prog_flag == true && *aa_flag == true {
    (rng.gen_range(-0.5, 0.5), rng.gen_range(-0.5, 0.5))
  } else {
    (0.0, 0.0)
  };
  let eyepos = *e + blur;
  let eyedir = *o + (x + r3) * *esx + (y + r4) * *esy - blur;
  Ray::new(&eyepos, &eyedir.normalize().unwrap())
}

fn pnm_header0(xr: &i32, yr: &i32, maxrad: &Flt) -> Vec<String> {
  vec![
    "P3".to_string(),
    format!("## max radiance = {}", maxrad),
    format!("{} {}", xr, yr),
    "255".to_string(),
  ]
}

fn radiance_to_rgb0(maxrad: &Flt, r: &Radiance) -> Rgb {
  let clip = |c: &Flt, d: &Flt| {
    let d2 = d / c;
    let r2: Flt = if d2 > 1.0 { 1.0 } else { d2 };
    f64::floor(r2.powf(GAMMA) * RGBMAX) as i32
  };
  Rgb(clip(maxrad, &r.0), clip(maxrad, &r.1), clip(maxrad, &r.2))
}

