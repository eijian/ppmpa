// camera

use std::collections::HashMap;
use std::f64;
use rand::Rng;

use super::ray::*;
use super::ray::algebra::*;
use super::ray::geometry::*;
use super::ray::optics::*;


pub struct Rgb(i32, i32, i32);

/*
impl fmt::Display for Rgb {
  fn fmt(&self.f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} {} {}", self.0, self.1, self.2)
  }
}
*/

pub struct Camera {
  //pub nphoton: i32,
  pub xreso: i32,
  pub yreso: i32,
  pub n_sample_photon: i32,
  pub progressive: bool,
  pub antialias: bool,
  pub use_classic_for_direct: bool,
  pub blur: bool,
  pub radius: Flt,
  pub max_radiance: Flt,
  pub iso_sens: Flt,
  pub shut_speed: Flt,
  pub focal_len: Flt,
  pub f_number: Flt,
  pub focus: Flt,
  pub pfilter: PhotonFilter,
  pub ambient: Radiance,
  pub eye_pos: Position3,
  pub photon_power: Flt,
  pub eye_dir: Direction3,
  pub screen_map: Vec<(Flt, Flt)>,
  pub origin: Position3,
  pub esx: Position3,
  pub esy: Position3,
  pub eex: Position3,
  pub eey: Position3,
}

pub const GAMMA: Flt  = 1.0 / 2.2;
pub const RGBMAX: Flt = 255.0;

//const DEFCONF: HashMap = HashMap

impl Camera {
  pub fn generate_ray(&self, (y, x): &(Flt, Flt)) -> Ray {
    let mut rng = rand::thread_rng();
    let blur_offset = if self.blur == true {
      let r1: Flt = rng.gen_range(-0.5, 0.5);
      let r2: Flt = rng.gen_range(-0.5, 0.5);
      r1 * self.eex + r2 * self.eey
    } else {
      Vector3::O
    };
    let (r3, r4) = if self.progressive == true && self.antialias == true {
      (rng.gen_range(-0.5, 0.5), rng.gen_range(-0.5, 0.5))
    } else {
      (0.0, 0.0)
    };
    let eyepos = self.eye_pos + blur_offset;
    let eyedir = self.origin + (x + r3) * self.esx + (y + r4) * self.esy - blur_offset;
    Ray::new(&eyepos, &eyedir.normalize().unwrap())
  }
  
  pub fn pnm_header(&self) -> Vec<String> {
    let ss = if self.shut_speed < 1.0 {
      format!("1/{}", 1.0 / self.shut_speed)
    } else {
      format!("{}", self.shut_speed)
    };
    vec![
      "P3".to_string(),
      format!("## max radiance = {}", self.max_radiance),
      format!("## image parameters = {}, F{}, ISO{}", ss, self.f_number, self.iso_sens),
      format!("{} {}", self.xreso, self.yreso),
      "255".to_string(),
    ]
  }
  
  pub fn radiance_to_rgb(&self, r: &Radiance) -> Rgb {
    let clip = |d: Flt| {
  //  let clip = |c: &Flt, d: &Flt| {
      let d2 = d / self.max_radiance;
      let r2 = if d2 > 1.0 { 1.0 } else { d2 };
      f64::floor(r2.powf(GAMMA) * RGBMAX) as i32
    };
    Rgb(clip(r.0), clip(r.1), clip(r.2))
  }  
}

const SENSOR_SIZE  : Flt = 35.0 / 1000.0;
const ISO_SENS     : Flt = 100.0;
const F_NUMBER     : Flt = 4.9;
const SHUTTER_SPEED: Flt = 1.0 / 250.0;

pub fn read_camera(_file: &str) -> Camera {
  let mut config = vec![
    ("x_resolution"   , "256"),
    ("y_resolution"   , "256"),
    ("progressive"    , "true"),
    ("antialias"      , "true"),
    ("use_classic"    , "true"),
    ("blur"           , "true"),
    ("estimate_radius", "0.2"),
    ("max_radiance"   , "0.01"),
    ("iso_sensitivity", "100"),    // ISO100 is default (enough photons)
    ("shutter_speed"  , "0.004"),  // unit is second
    ("focal_length"   , "50.0"),   // unit is 'mm'
    ("f_number"       , "4.0"),
    ("focus"          , "7.0"),
    ("photon_filter"  , "PF:None"),
    ("ambient"        , "RAD[0.0,0.0,0.0]"),  // ambient light intensity
    ("eye_position"   , "V3[1.0,2.0,-4.5]"),  // center of a camera diaphragm
    ("target_position", "V3[0.0,1.0,0.0]"),   // center of a screen
    ("upper_direction", "V3[0.0,1.0,0.0]"),
  ].into_iter().collect::<HashMap<_, _>>();


  //let target = Vector3::new_pos(0.0, 2.0, 0.0);
  let xreso      = param_int(&config, "x_resolution");
  let yreso      = param_int(&config, "y_resolution");
  let prog_flag  = param_bool(&config, "progressive");
  let aa_flag    = param_bool(&config, "antialias");
  let uc_flag    = param_bool(&config, "use_classic");
  let blur_flag  = param_bool(&config, "blur");
  let radius     = param_float(&config, "estimate_radius");
  let max_rad    = param_float(&config, "max_radiance");
  let iso_sens   = param_float(&config, "iso_sensitivity");
  let shut_speed = param_float(&config, "shutter_speed");
  let focal_len  = param_float(&config, "focal_length") / 1000.0;
  let f_number   = param_float(&config, "f_number");
  let focus      = param_float(&config, "focus");
  let pf         = config.get("photon_filter").unwrap().parse::<PhotonFilter>().unwrap();
  let ambient    = param_rad(&config, "ambient");
  let eyepos     = param_vec3(&config, "eye_position");
  let target     = param_vec3(&config, "target_position");
  let upper      = param_vec3(&config, "upper_direction");

  let _ez = (target - eyepos).normalize().unwrap();
  let _ex = upper.cross(&_ez).normalize().unwrap();
  let _ey = _ex.cross(&_ez).normalize().unwrap();

  let _step = (focus * SENSOR_SIZE / focal_len) / xreso as Flt;
  let esx = _step * _ex;
  let esy = _step * _ey;
  let _ea = focal_len / f_number;
  let eex = _ea * _ex;
  let eey = _ea * _ey;
  let _lx = (xreso / 2) as Flt;
  let _ly = (yreso / 2) as Flt;
  let orig = focus * _ez - (_lx - 0.5) * esx - (_ly - 0.5) * esy;
  let ppower = if blur_flag {
    iso_sens / ISO_SENS * F_NUMBER / f_number * shut_speed / SHUTTER_SPEED
  } else {
    1.0
  };

  let mut smap: Vec<(Flt, Flt)> = vec![];
  for y in 0..yreso {
    for x in 0..xreso {
      smap.push((y as Flt, x as Flt));
    }
  }

  let cam = Camera {
    //nphoton: 500000,
    xreso: xreso,
    yreso: yreso,
    n_sample_photon: 500,
    progressive: prog_flag,
    antialias: aa_flag,
    use_classic_for_direct: uc_flag,
    blur: blur_flag,
    radius: radius * radius,            // squared radius 
    max_radiance: max_rad,
    iso_sens: iso_sens,
    shut_speed: shut_speed,
    focal_len: focal_len,
    f_number: f_number,
    focus: focus,
    pfilter: pf,
    ambient: ambient, //Radiance(0.001, 0.001, 0.001), //
    eye_pos: eyepos,
    photon_power: ppower,
    eye_dir: (target - eyepos).normalize().unwrap(),    
    screen_map: smap,
    origin: orig,
    esx: esx,
    esy: esy,
    eex: eex,
    eey: eey,
  };
  cam
}

pub fn rgb_to_string(c: &Rgb) -> String {
  format!("{} {} {}", c.0, c.1, c.2)
}

pub fn radiance_to_string(r: &Radiance) -> String {
  format!("{:e} {:e} {:e}", r.0, r.1, r.2)
}

pub fn rgb_to_radiance(cam: &Camera, c: &Rgb) -> Radiance {
  let mag = cam.max_radiance / RGBMAX;
  Radiance(c.0 as Flt * mag, c.1 as Flt * mag, c.2 as Flt * mag)
}

//--------------------
// private

fn param_int(config: &HashMap<&str, &str>, p: &str) -> i32 {
  config.get(p).unwrap().parse::<i32>().unwrap()
}

fn param_bool(config: &HashMap<&str, &str>, p: &str) -> bool {
  config.get(p).unwrap().parse::<bool>().unwrap()
}

fn param_float(config: &HashMap<&str, &str>, p: &str) -> Flt {
  config.get(p).unwrap().parse::<Flt>().unwrap()
}

fn param_vec3(config: &HashMap<&str, &str>, p: &str) -> Vector3 {
  config.get(p).unwrap().parse::<Vector3>().unwrap()
}

fn param_rad(config: &HashMap<&str, &str>, p: &str) -> Radiance {
  config.get(p).unwrap().parse::<Radiance>().unwrap()
}


/*
fn make_generate_ray(aa_flag: &bool, prog_flag: &bool, epos: &Position3,
  target: &Direction3, xr: &i32, yr: &i32, udir: &Direction3, fd: &Flt, fl: &Flt)
  -> fn(&(Flt, Flt)) -> Ray {
  let ez = (*target - *epos).normalize().unwrap();
  let ex = udir.cross(&ez).normalize().unwrap();
  let ey = ex.cross(&ez).normalize().unwrap();

  let step = (*fd * 0.035 / *fl) / *xr as Flt;
  let esx = step * ex;
  let esy = step * ey;
  let fnum: Flt = 4.0;
  let ea = *fl / fnum;
  let eex = ea * ex;
  let eey = ea * ey;
  let lx = (xr / 2) as Flt;
  let ly = (yr / 2) as Flt;
  let orig = *fd * ez - (lx - 0.5) * esx - (ly - 0.5) * esy;
  let blur_flag = false;
  let f: Fn(&(Flt, Flt)) -> Ray = |p| generate_ray0(aa_flag, prog_flag, &blur_flag, epos, &orig, &esx, &esy, &eex, &eey, p);
  f
}
*/
