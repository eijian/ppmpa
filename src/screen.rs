// screen

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
  pub blur: bool,
  pub origin: Position3,
  pub esx: Position3,
  pub esy: Position3,
  pub eex: Position3,
  pub eey: Position3,
}

pub const GAMMA: Flt  = 1.0 / 2.2;
pub const RGBMAX: Flt = 255.0;

//const DEFCONF: HashMap = HashMap

impl Screen {
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
    vec![
      "P3".to_string(),
      format!("## max radiance = {}", self.max_radiance),
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


pub fn read_screen(file: &str) -> Screen {
  let target = Vector3::new_pos(0.0, 2.0, 0.0);
  let eyepos = Vector3::new_pos(0.0, 2.0, -4.5);
  let upper  = Vector3::EY;
  let focus = 7.0;
  let focallen: Flt = 50.0 / 1000.0;
  let fnumber: Flt = 5.0;
  let xreso = 256;
  let yreso = 256;
  let aa_flag = true;
  let prog_flag = false;
  let blur_flag = false;

  let _ez = (target - eyepos).normalize().unwrap();
  let _ex = upper.cross(&_ez).normalize().unwrap();
  let _ey = _ex.cross(&_ez).normalize().unwrap();

  let _step = (focus * 0.035 / focallen) / xreso as Flt;
  let esx = _step * _ex;
  let esy = _step * _ey;
  let _fnum: Flt = 4.0;
  let _ea = focallen / fnumber;
  let eex = _ea * _ex;
  let eey = _ea * _ey;
  let _lx = (xreso / 2) as Flt;
  let _ly = (yreso / 2) as Flt;
  let orig = focus * _ez - (_lx - 0.5) * esx - (_ly - 0.5) * esy;

  let mut smap: Vec<(Flt, Flt)> = vec![];
  for y in 0..yreso {
    for x in 0..xreso {
      smap.push((y as Flt, x as Flt));
    }
  }

  let scr = Screen {
    nphoton: 1000000,
    progressive: prog_flag,
    xreso: xreso,
    yreso: yreso,
    antialias: aa_flag,
    n_sample_photon: 500,
    use_classic_for_direct: true,
    radius: 0.2,
    pfilter: PhotonFilter::Gauss,
    ambient: Radiance::RADIANCE0, //Radiance(0.001, 0.001, 0.001), //
    max_radiance: 0.01,
    eye_pos: eyepos,
    eye_dir: (target - eyepos).normalize().unwrap(),    
    focus: focus,
    screen_map: smap,
    blur: blur_flag,
    origin: orig,
    esx: esx,
    esy: esy,
    eex: eex,
    eey: eey,
  };
  scr
}

pub fn rgb_to_string(c: &Rgb) -> String {
  format!("{} {} {}", c.0, c.1, c.2)
}

pub fn radiance_to_string(r: &Radiance) -> String {
  format!("{:e} {:e} {:e}", r.0, r.1, r.2)
}

pub fn rgb_to_radiance(scr: &Screen, c: &Rgb) -> Radiance {
  let mag = scr.max_radiance / RGBMAX;
  Radiance(c.0 as Flt * mag, c.1 as Flt * mag, c.2 as Flt * mag)
}

//--------------------
// private functions

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
