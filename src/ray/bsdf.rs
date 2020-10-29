// BSDF
//   Reference URLs
//   https://www.slideshare.net/teppeikurita/brdf-196782059
//

use super::*;
use super::algebra::*;
use super::material::*;
use super::optics::*;
use super::physics::*;

const ONE_PI: Flt = 1.0 / f64::consts::PI;
const SR_HALF: Flt = 1.0 / (2.0 * f64::consts::PI);

pub enum BSDF {
  Simple,
  Simple2,
  LambertBlinn,
  DisneyBRDF,
  Brady,
}

pub fn bsdf(nvec: &Direction3, rdir: &Direction3, tdir: &Direction3,
            mate: &Material, cos0: &Flt, ior0: &Flt, ior1: &Flt,
            di: &Radiance, si: &Radiance, ti: &Radiance) -> Radiance {
  //let mate = is.mate;
  let f  = reflection_index(&mate.specular_refl, cos0);
  let f2 = -f;

  let model = BSDF::Simple;

  match model {
    BSDF::Simple         => {
      mate.emittance * SR_HALF +
      mate.diffuseness         * (mate.reflectance * ONE_PI * *di) +
      (1.0 - mate.diffuseness) * (f * *si + (1.0 - mate.metalness) * f2 * *ti)
    },
    BSDF::Simple2        => {
      mate.emittance * SR_HALF +
      (1.0 - mate.metalness) * (mate.reflectance * ONE_PI * *di + f * *si + f2 * *ti) +
      mate.metalness         * (f * *si)
    },
    BSDF::LambertBlinn => {
      Radiance::RADIANCE0
    },
    BSDF::DisneyBRDF   => {
      Radiance::RADIANCE0
    },
    BSDF::Brady        => {
      Radiance::RADIANCE0
    },
  }
}

pub fn select_diffuse(mate: &Material) -> bool {
  let m = mate.metalness;
  russian_roulette(&[m]) > 0
}


// private methods

fn reflection_index(col: &Color, c: &Flt) -> Color {
  let c2 = (1.0 - *c).powf(5.0);
  Color(col.0 + (1.0 - col.0) * c2, col.1 + (1.0 - col.1) * c2, col.2 + (1.0 - col.2) * c2)
}


//



