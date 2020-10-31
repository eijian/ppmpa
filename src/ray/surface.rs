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
  Simple {
    reflectance:   Color,
    specular_refl: Color,
    diffuseness:   Flt,
    metalness:     Flt,
    roughness:     Flt,
  },
  LambertBlinn,
  DisneyBRDF,
  Brady,
}

impl Surface {

  pub fn reflect(&self, cos: &Flt) -> bool {
    match self {
      Surface::Nothing => false,
      Surface::Simple {
        reflectance,
        specular_refl,
        diffuseness,
        metalness,
        roughness,
      } => {
        (*diffuseness == 1.0 ||
         (*cos == 1.0 && *specular_refl == Color::BLACK)
        ) == false
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
      } => {
        (*cos == 0.0 && *specular_refl == Color::WHITE) == false
      },
      _ => false,
    }
  }

  pub fn bsdf(&self, nvec: &Direction3, rdir: &Direction3, tdir: &Direction3,
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
      } => {
        let f  = reflection_index(specular_refl, cos0);
        let f2 = -f;
        *diffuseness         * (*reflectance * ONE_PI * *di) +
        (1.0 - *diffuseness) * (f * *si + (1.0 - *metalness) * f2 * *ti)
      },
      Surface::LambertBlinn => {
        Radiance::RADIANCE0
      },
      Surface::DisneyBRDF   => {
        Radiance::RADIANCE0
      },
      Surface::Brady        => {
        Radiance::RADIANCE0
      },
    }
  }


  pub fn select_diffuse(&self) -> bool {
    match self {
      Surface::Simple {
        reflectance,
        specular_refl,
        diffuseness,
        metalness,
        roughness,
      } => {
        let m = *metalness;
        russian_roulette(&[m]) > 0
      },
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
      } => reflectance.select_wavelength(wl),
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
      } => specular_refl.select_wavelength(wl),
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
      } => *diffuseness,
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
        } => {
          match roughness {
            0.0 => v,
            _   => reflection_glossy(n, &v, roughness),
          }
        },
        _ => v,
      };
      if c < 0.0 {
        (r, -c)
      } else {
        (r, c)
      }
    }
    None => (*n, 0.0),
  }
}

pub fn specular_refraction(ior: &Flt, c0: &Flt, ed: &Direction3, n: &Direction3) -> Direction3 {
  let r = 1.0 / (ior * ior) + c0 * c0 - 1.0;
  let a = c0 - f64::sqrt(r);
  let n2 = if ed.dot(n) > 0.0 { -(*n) } else { *n };
  let v = (*ior * (*ed + a * n2)).normalize();
  match v {
    Some(t) if r >= 0.0 => t,
    _                   => Vector3::O,
  }
}


// glossyな表面の反射ベクトルの求め方
//   http://www.raytracegroundup.com/downloads/Chapter25.pdf
//   https://cg.informatik.uni-freiburg.de/course_notes/graphics2_08_renderingEquation.pdf
//   https://graphics.cg.uni-saarland.de/courses/ris-2018/slides/09_BRDF_LightSampling.pdf
//   x = cos(2 pi xi2) sqrt(1 - (1 - xi1)^2)
//   y = 1 - xi1
//   z = sin(2 pi xi2) sqrt(1 - (1 - xi1)^2)
//     where xi1 = cos (w.r) ^ 10 ^ (5 x (1 - roughness))
pub fn reflection_glossy(nvec: &Direction3, rvec: &Direction3, rough: &Flt) -> Direction3 {
  let uvec = Vector3::new(0.00424, 1.0, 0.00764).cross(&rvec).normalize().unwrap();
  let vvec = uvec.cross(rvec);
  let mut rng = rand::thread_rng();
  let pw = 1.0 / (10.0_f64.powf(5.0 * (1.0 - rough)) + 1.0);
  let xi1 = (rng.gen_range(0.0, 1.0) as Flt).powf(pw);
  let xi2 = 2.0 * f64::consts::PI * rng.gen_range(0.0, 1.0);

  let x = f64::cos(xi2) * f64::sqrt(1.0 - xi1 * xi1);
  let y = xi1;
  let z = f64::sin(xi2) * f64::sqrt(1.0 - xi1 * xi1);

  let mut wi = x * uvec + y * *rvec + z * vvec;
  if nvec.dot(&wi) < 0.0 {
    wi = -x * uvec + y * *rvec - z * vvec;
  }
  wi.normalize().unwrap()
}

// private methods

fn reflection_index(col: &Color, c: &Flt) -> Color {
  let c2 = (1.0 - *c).powf(5.0);
  Color(col.0 + (1.0 - col.0) * c2, col.1 + (1.0 - col.1) * c2, col.2 + (1.0 - col.2) * c2)
}


//



