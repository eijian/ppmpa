// surface
//   Reference URLs
//   https://www.slideshare.net/teppeikurita/brdf-196782059
//

use rand::Rng;

use super::*;
use super::algebra::*;
use super::optics::*;
use super::physics::*;

const ONE_PI: Flt = 1.0 / f64::consts::PI;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Surface {
  Nothing,
  // My original model
  Simple {
    reflectance:   Color,
    specular_refl: Color,
    diffuseness:   Flt,
    metalness:     Flt,
    roughness:     Flt,
    // calculate values
    density_pow:   Flt,
  },
  // Torrance-Sparrow model
  TS {
    albedo_diff: Color,
    albedo_spec: Color,
    scatterness: Flt,
    metalness:   Flt,   // 0.0:dielectric, 1.0: metal
    roughness:   Flt,
    // calculate values
    density_pow: Flt,
    alpha:       Flt,
  },
  DisneyBRDF,
  Brady,
}

impl Surface {

  pub fn new_simple(refl: &Color, spec: &Color, diff: &Flt, meta: &Flt, rough: &Flt) -> Surface {
    Surface::Simple {
      reflectance:   *refl,
      specular_refl: *spec,
      diffuseness:   *diff,
      metalness:     *meta,
      roughness:     *rough,
      density_pow:   1.0 / (10.0_f64.powf(5.0 * (1.0 - f64::sqrt(*rough))) + 1.0),
    }
  }

  pub fn new_ts(al_diff: &Color, al_spec: &Color, scat: &Flt, meta: &Flt, rough: &Flt) -> Surface {
    Surface::TS {
      albedo_diff: *al_diff,
      albedo_spec: *al_spec,
      scatterness: *scat,
      metalness:   *meta,
      roughness:   *rough,
      density_pow: 1.0 / (10.0_f64.powf(5.0 * (1.0 - f64::sqrt(*rough))) + 1.0),
      alpha:       *rough * *rough * *rough * *rough,
    }
  }

  pub fn reflect(&self, cos: &Flt) -> bool {
    match self {
      Surface::Nothing => false,
      Surface::Simple {
        reflectance,
        specular_refl,
        diffuseness,
        metalness,
        roughness,
        density_pow,
      } => {
        (*diffuseness == 1.0 ||
         (*cos == 1.0 && *specular_refl == Color::BLACK)
        ) == false
      },
      Surface::TS {
        albedo_diff,
        albedo_spec,
        scatterness,
        metalness,
        roughness,
        density_pow,
        alpha,
      } => {
        match metalness {
          0.0 => true,
          1.0 => *albedo_spec != Color::BLACK,
          _   => true,
        }
      },
      _ => false,
    }
  }

  pub fn refract(&self, cos: &Flt) -> bool {
    match self {
      Surface::Nothing => false,
      Surface::Simple {
        reflectance,
        specular_refl,
        diffuseness,
        metalness,
        roughness,
        density_pow,
      } => {
        (*cos == 0.0 && *specular_refl == Color::WHITE) == false
      },
      Surface::TS {
        albedo_diff,
        albedo_spec,
        scatterness,
        metalness,
        roughness,
        density_pow,
        alpha,
      } => {
        if *metalness == 0.0 {
          if *scatterness < 1.0 && *albedo_diff != Color::BLACK {
            return true
          }
        }
        false
      },
      _ => false,
    }
  }

  pub fn bsdf(&self, nvec: &Direction3, edir: &Direction3, rdir: &Direction3, tdir: &Option<Direction3>,
              cos0: &Flt, ior: &Flt, di: &Radiance, si: &Radiance, ti: &Radiance)
             -> Radiance {
    //let mate = is.mate;

    match self {
      Surface::Nothing => Radiance::RADIANCE0,
      Surface::Simple {
        reflectance,
        specular_refl,
        diffuseness,
        metalness,
        roughness,
        density_pow,
      } => {
        let f  = reflection_index(specular_refl, cos0);
        let f2 = -f;
        *diffuseness         * (*reflectance * ONE_PI * *di) +
        (1.0 - *diffuseness) * (f * *si + (1.0 - *metalness) * f2 * *ti)
      },
      Surface::TS {
        albedo_diff,
        albedo_spec,
        scatterness,
        metalness,
        roughness,
        density_pow,
        alpha,
      } => {
        let lvec = *rdir;
        let vvec = -*edir;
        let hvec = (lvec + vvec).normalize().unwrap();
        let cos_h = hvec.dot(&vvec);
        //let f = reflection_index(albedo_spec, &cos_h);
        let f = reflection_index(albedo_spec, &cos0);
        let f2 = -f;  // (1 - f)
        let i_de = match metalness {
          0.0 => f2 * *albedo_diff * (
            *scatterness         * ONE_PI * *di
            // 屈折率の異なる物質に入射した光の輝度は1/eta^2となるらしい
            //+ (1.0 - *scatterness) * 1.0 / (ior * ior) * *ti
            + (1.0 - *scatterness) * *ti
          ),
          _   => Radiance::RADIANCE0,
        };
        /*
        let cos_v = nvec.dot(&vvec);   // (v.n)
        //let cos_v = *cos0;             // (v.n)
        let cos_l = nvec.dot(&lvec);   // (l.n)
        let cos_n = nvec.dot(&hvec);   // (h.n)
        //println!("a:{}/cos_n:{}/cos_v:{}/cos_l:{}", alpha, cos_n, cos_v, cos_l);
        let cos_sq = cos_n * cos_n;
        let d0 = cos_sq * (alpha - 1.0) + 1.0;
        let d = (alpha / f64::consts::PI) / (d0 * d0);
        let lam_v = -1.0 + f64::sqrt(1.0 + alpha * (1.0 / (cos_v * cos_v) - 1.0)) / 2.0;
        let lam_l = -1.0 + f64::sqrt(1.0 + alpha * (1.0 / (cos_l * cos_l) - 1.0)) / 2.0;
        let g = (1.0 / (1.0 + lam_l)) * (1.0 / (1.0 + lam_v));
        //let i_mt = (d * g / (4.0 * cos_l * cos_v)) * f * *si;
        // そもそもsiにTSモデルなどは適用しない？
        */
        let i_mt = f * *si;
        //let i_mt = f * *si + (f2 * ior * ior) * *ti;
        i_de + i_mt
      },
      Surface::DisneyBRDF   => {
        Radiance::RADIANCE0
      },
      Surface::Brady        => {
        Radiance::RADIANCE0
      },
    }
  }

  // OUT: dir  next ray direction. if dir is None, the photon is absorbed.
  //      T/F  true=reflection, false=refraction

  pub fn next_direction(&self, eta: &Flt, nvec: &Direction3, vvec: &Direction3, wl: &Wavelength) -> Option<(Direction3, bool)> {
    let (rdir0, cos1) = specular_reflection(nvec, vvec);
    let rdir = reflection_glossy(nvec, &rdir0, &self.power_glossy());
    let hvec = (rdir - *vvec).normalize().unwrap();
    let (tdir, cos2) = specular_refraction(&hvec, vvec, eta);
    let cos = if cos1 < cos2 { cos1 } else { cos2 };
    match self {
      Surface::Simple {
        reflectance,
        specular_refl,
        diffuseness,
        metalness,
        roughness,
        density_pow,
      } => {
        None
      },
      Surface::TS {
        albedo_diff,
        albedo_spec,
        scatterness,
        metalness,
        roughness,
        density_pow,
        alpha,
      } => {
        let f = schlick(&albedo_spec.wavelength(&wl), &cos);
        // 鏡面反射
        if russian_roulette(&[f]) == 0 {
          return Some((rdir, true))
        }
        // 吸収
        if russian_roulette(&[albedo_diff.wavelength(&wl)]) == 1 {
          return None
        }
        // 拡散反射
        if russian_roulette(&[*scatterness]) == 0 {
          return Some((diffuse_reflection(nvec), true))
        }
        // 鏡面透過
        match tdir {
          Some(tdir) => Some((tdir, false)),
          _ => None,
        }
      },
      _ => None,
    }
  }

  pub fn select_diffuse(&self, cos: &Flt, wl: &Wavelength) -> bool {
    match self {
      Surface::Simple {
        reflectance,
        specular_refl,
        diffuseness,
        metalness,
        roughness,
        density_pow,
      } => {
        let m = *metalness;
        russian_roulette(&[m]) > 0
      },
      Surface::TS {
        albedo_diff,
        albedo_spec,
        scatterness,
        metalness,
        roughness,
        density_pow,
        alpha,
      } => {
        let f = schlick(&albedo_spec.wavelength(wl), &cos);
        russian_roulette(&[f]) > 0
      },
      _ => true,
    }
  }

  pub fn store_photon(&self) -> bool {
    match self {
      Surface::Simple {
        reflectance,
        specular_refl,
        diffuseness,
        metalness,
        roughness,
        density_pow,
      } => *diffuseness > 0.0,
      Surface::TS {
        albedo_diff,
        albedo_spec,
        scatterness,
        metalness,
        roughness,
        density_pow,
        alpha,
      } => *metalness != 1.0 && *scatterness != 0.0,
      _ => true,     
    }
  }

  pub fn albedo_diff(&self, wl: &Wavelength) -> Flt {
    match self {
      Surface::Simple {
        reflectance,
        specular_refl,
        diffuseness,
        metalness,
        roughness,
        density_pow,
      } => reflectance.wavelength(wl),
      Surface::TS {
        albedo_diff,
        albedo_spec,
        scatterness,
        metalness,
        roughness,
        density_pow,
        alpha,
      } => albedo_diff.wavelength(wl),
      _ => 0.0,
    }
  }

  pub fn albedo_spec(&self, wl: &Wavelength) -> Flt {
    match self {
      Surface::Simple {
        reflectance,
        specular_refl,
        diffuseness,
        metalness,
        roughness,
        density_pow,
      } => specular_refl.wavelength(wl),
      Surface::TS {
        albedo_diff,
        albedo_spec,
        scatterness,
        metalness,
        roughness,
        density_pow,
        alpha,
      } => albedo_spec.wavelength(wl),
      _ => 0.0,
    }
  }

  pub fn roughness(&self) -> Flt {
    match self {
      Surface::Simple {
        reflectance,
        specular_refl,
        diffuseness,
        metalness,
        roughness,
        density_pow,
      } => *diffuseness,
      Surface::TS {
        albedo_diff,
        albedo_spec,
        scatterness,
        metalness,
        roughness,
        density_pow,
        alpha,
      } => *roughness,
      _ => 0.0,
    }
  }

  pub fn power_glossy(&self) -> Flt {
    match self {
      Surface::Simple {
        reflectance,
        specular_refl,
        diffuseness,
        metalness,
        roughness,
        density_pow,
      } => *density_pow,
      Surface::TS {
        albedo_diff,
        albedo_spec,
        scatterness,
        metalness,
        roughness,
        density_pow,
        alpha,
      } => *density_pow,
      _ => 0.0,
    }
  }
}

// utility functions
//

pub fn diffuse_reflection(n: &Direction3) -> Direction3 {
  //let d = generate_random_dir();
  let d = generate_random_dir_by_angle();
  let c = n.dot(&d);
  if c > 0.0 {
    d
  } else {
    -d
  }
}

/*
pub fn specular_reflection(n: &Direction3, e: &Direction3, sf: &Surface) -> (Direction3, Flt) {
  let c = e.dot(n);
  let v = (*e - (2.0 * c) * *n).normalize();
  match v {
    Some(v) => {
      let r = match sf {
        Surface::Simple {
          reflectance,
          specular_refl,
          diffuseness,
          metalness,
          roughness,
          density_pow,
        } => {
          match roughness {
            0.0 => v,
            _   => reflection_glossy(n, &v, density_pow),
          }
        },
        Surface::TS {
          albedo_diff,
          albedo_spec,
          scatterness,
          metalness,
          roughness,
          density_pow,
          alpha,
        }    => {
          match roughness {
            0.0 => v,
            _   => reflection_glossy(n, &v, density_pow),
          }
        },
        _ => v,
      };
      if c < 0.0 {
        (r, -c)
      } else {
        (r, c)
      }
    },
    None => (*n, 0.0),
  }
}

pub fn specular_refraction(ior: &Flt, c0: &Flt, ed: &Direction3, n: &Direction3, sf: &Surface) -> Option<Direction3> {
  let r = 1.0 / (ior * ior) + c0 * c0 - 1.0;
  if r < 0.0 {
    return None
  }
  let a = c0 - f64::sqrt(r);

  let n2 = match sf {
    Surface::Simple {
      reflectance,
      specular_refl,
      diffuseness,
      metalness,
      roughness,
      density_pow,
    } => {
      if *roughness == 0.0 {
        *n
      } else {
        let r = reflection_glossy(n, ed, density_pow);
        (r - *ed).normalize().unwrap()
      }
    },
    Surface::TS {
      albedo_diff,
      albedo_spec,
      scatterness,
      metalness,
      roughness,
      density_pow,
      alpha,
    } => {
      if *roughness == 0.0 {
        *n
      } else {
        let r = reflection_glossy(n, ed, density_pow);
        (r - *ed).normalize().unwrap()
      }
    },
    _ => *n,
  };
  (*ior * (*ed + a * n2)).normalize()
}
*/

// private methods

fn reflection_index(col: &Color, c: &Flt) -> Color {
  let c2 = (1.0 - *c).powf(5.0);
  Color(col.0 + (1.0 - col.0) * c2, col.1 + (1.0 - col.1) * c2, col.2 + (1.0 - col.2) * c2)
}


//



