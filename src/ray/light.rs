// light

use std::fmt;
use rand::Rng;

use super::*;
use super::algebra::*;
use super::geometry::*;
use super::physics::*;
use super::optics::*;

pub type Flux = Flt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Light {
  PointLight {
    color: Color,
    flux: Flux,
    pos: Position3
  },
  ParallelogramLight {
    color: Color,
    flux: Flux,
    pos: Position3,
    nvec: Direction3,
    dir1: Direction3,
    dir2: Direction3
  },
  SunLight {
    color: Color,
    flux: Flux,
    pos: Position3,
    nvec: Direction3,
    dir1: Direction3,
    dir2: Direction3,
    dir: Direction3,
  },
}

impl fmt::Display for Light {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let res = match self {
      Light::PointLight {color, flux, pos}
        => write!(f, "[{},{},{}]", color, flux, pos),
      Light::ParallelogramLight {color, flux, pos, nvec, dir1, dir2}
        => write!(f, "[{},{},{},{},{},{}]", color, flux, pos, nvec, dir1, dir2),
      Light::SunLight {color, flux, pos, nvec, dir1, dir2, dir}
        => write!(f, "[{},{},{},{},{},{},{}]", color, flux, pos, nvec, dir1, dir2, dir),
    };
    res
  }
}

impl Light {
  pub fn flux(&self) -> Flt {
    match self {
      Light::PointLight {color: _, flux, pos: _}
        => *flux,
      Light::ParallelogramLight {color: _, flux, pos: _, nvec: _, dir1: _, dir2: _}
        => *flux,
      Light::SunLight {color: _, flux, pos: _, nvec: _, dir1: _, dir2: _, dir: _}
        => *flux,
    }
  }

  pub fn generate_photon(&self) -> Photon {
    match self {
      Light::PointLight {color, flux:_, pos}
        => Photon::new(&select_wavelength(color), &Ray::new(pos, &generate_random_dir())),
      Light::ParallelogramLight {color, flux:_, pos, nvec, dir1, dir2}
        => {
          let w = select_wavelength(color);
          let mut rng = rand::thread_rng();
          let t1 = rng.gen_range(0.0, 1.0);
          let t2 = rng.gen_range(0.0, 1.0);
          let d = diffuse_reflection(nvec);
          let r = Ray::new(&(*pos + t1 * *dir1 + t2 * *dir2), &d);
          Photon::new(&w, &r)
        },
      Light::SunLight {color, flux:_, pos, nvec:_, dir1, dir2, dir}
        => {
          let w = select_wavelength(color);
          let mut rng = rand::thread_rng();
          let t1 = rng.gen_range(0.0, 1.0);
          let t2 = rng.gen_range(0.0, 1.0);
          let r = Ray::new(&(*pos + t1 * *dir1 + t2 * *dir2), dir);
          Photon::new(&w, &r)
        }
    }
  }

  pub fn get_direction(&self, p: &Position3) -> Vec<Direction3> {
    let vs: Vec<Direction3> = match self {
      Light::PointLight {color:_, flux:_, pos}
        => vec![*pos - *p],
      Light::ParallelogramLight {color:_, flux:_, pos, nvec, dir1, dir2}
        /* little faster
        => {
          let mut vs0: Vec<Direction3> = vec![];
          for (tx, ty) in &TSS {
            let v = gen_pos(pos, dir1, dir2, tx, ty) - *p;
            if nvec.dot(&v) < 0.0 { vs0.push(v); }
          }
          vs0
        }
        */
        => TSS.iter()
              .map(|(tx, ty)| gen_pos(pos, dir1, dir2, tx, ty) - *p)
              .filter(|d| nvec.dot(d) < 0.0).collect::<Vec<Direction3>>(),
      Light::SunLight {color:_, flux:_, pos, nvec, dir1, dir2, dir}
        => {
          let d = *pos - *p;
          let cos0 = nvec.dot(&d);
          if cos0 > 0.0 {
            vec![]
          } else {
            let dt2 = -(*dir);
            let res = method_moller(&2.0, pos, dir1, dir2, p, &dt2);
            if res == None {
              vec![]
            } else {
              vec![res.unwrap().2 * dt2]
            }
          }
        },
    };
    vs
  }

  pub fn get_radiance(&self, ds: &Vec<Flt>) -> Vec<Radiance> {
    let mut rs = vec![Radiance::RADIANCE0];
    for d in ds {
      match self {
        Light::PointLight {color, flux, ..}
          => {
            let l0 = flux / (PI4 * d);
            rs.push(Radiance(color.0 * l0, color.1 * l0, color.2 * l0))
          },
        Light::ParallelogramLight {color, flux, ..}
          => {
            let l0 = (2.0 * flux * PARA_DIV * PARA_DIV) / (PI4 * d);
            rs.push(Radiance(color.0 * l0, color.1 * l0, color.2 * l0))
          },
        Light::SunLight {color, flux, ..}
          => rs.push(Radiance(color.0 * flux, color.1 * flux, color.2 * flux)),
      }
    }
    rs
  }
}

fn gen_pos(pos: &Position3, dir1: &Direction3, dir2: &Direction3, x: &Flt, y: &Flt) -> Position3 {
  *pos + *x * *dir1 + *y * *dir2
}

fn select_wavelength(c: &Color) -> Wavelength {
  let mut rng = rand::thread_rng();
  c.decide_wavelength(rng.gen_range(0.0, 1.0))
}

const PARA_DIV: Flt = 0.2;
//const TS: [Flt; 5] = [0.1, 0.3, 0.5, 0.7, 0.9];
const TSS: [(Flt, Flt); 25] = [
  (0.1, 0.1), (0.1, 0.3), (0.1, 0.5), (0.1, 0.7), (0.1, 0.9),
  (0.3, 0.1), (0.3, 0.3), (0.3, 0.5), (0.3, 0.7), (0.3, 0.9),
  (0.5, 0.1), (0.5, 0.3), (0.5, 0.5), (0.5, 0.7), (0.5, 0.9),
  (0.7, 0.1), (0.7, 0.3), (0.7, 0.5), (0.7, 0.7), (0.7, 0.9),
  (0.9, 0.1), (0.9, 0.3), (0.9, 0.5), (0.9, 0.7), (0.9, 0.9)
  ];






