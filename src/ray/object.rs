// Object module

use super::{Flt};
use super::algebra::{Direction3, SurfacePoint};
use super::geometry::{ShapeFunc, Shape};
use super::light::{LightSpec, RadEstimation};
use super::surface::{Surface};
use super::optics::{Photon, Radiance};


// Object --------------

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Object<'a> {
  shape  : Shape,
  surface: Surface<'a>,
  light  : Option<LightSpec>,  // 発光（光源も物体の一つ）情報、光源でなければNone
}

impl<'a> Object<'a> {

  #[inline(always)]
  pub fn new(shape: &Shape, surface: &'a Surface, lgtspec: &Option<LightSpec>) -> Self {
    Object {shape: *shape, surface: *surface, light: *lgtspec}
  }

  #[inline(always)]
  pub fn is_light(&self) -> bool {
    if let Some(_l) = self.light {
      true
    } else {
      false
    }
  }

  // emittance
  //   IN: sp 視線と物体表面が交わる点（座標＋法線ベクトル）
  //       vvec 視線ベクトル（物体へ向かう方向）
  pub fn emittance(&self, sp: &SurfacePoint, vvec: &Direction3) -> Radiance {
    match self.light {
      Some(l) => l.emittance(sp, vvec),
      None => Radiance::ZERO,
    }
  }

  // flux: 物体の放射束 [W]

  pub fn flux(&self) -> Flt {
    match self.light {
      Some(ls) => self.shape.surface_area() * ls.radiosity(),
      None => 0.0,
    }
  }

  // emit_photons: 物体の放射束を元に、エネルギー e のフォトンを複数放射する
  //   IN:
  //     e 光子エネルギー[W] ※ 物体の放射束に対しエネルギーeの光子を放出したい＝放射する光子数 n
  //   OUT:
  //     光子(n個)

  pub fn emit_photons(&self, e: Flt) -> Vec<(Photon, RadEstimation)> {
    let mut photons: Vec<(Photon, RadEstimation)> = Vec::new();
    if let Some(ls) = self.light {
      let flux = self.flux();
      let n = (flux / e) as i32;
      for _i in 0..n {
        photons.push(ls.generate_photon(&self.shape.random_point()));
      }
    }
    photons
  }

}

#[cfg(test)]
mod tests_object {
  use super::*;

  use crate::ray::{PI4};
  use crate::ray::algebra::{Ray, Vector, Vector3};
  use crate::ray::light::{InOut};
  use crate::ray::material::{Material};
  use crate::ray::optics::{Color};
  use crate::ray::surface::{Mapping, Surface};
  
  #[test]
  fn test_self() {
    // new, is_light
    let sp = Shape::new_sphere(&Vector3::O, 0.5);
    let col = Color::new_by_kelvin(5000.0);
    let ls = LightSpec::new(&col, 1500.0, 0.0, RadEstimation::Photon, InOut::Out);
    let m1 = Material::new(&Color::WHITE, &Color::BLACK,
                                    &Color(0.13, 0.45, 1.43), &Color(4.06, 2.42, 1.95), 0.0, 1.0);
    let mt1 = Mapping::new_solid(&m1);
    let sf1 = Surface::new(0.0, mt1);
    let ob1 = Object::new(&sp, &sf1, &Some(ls));

    assert_eq!(ob1.light, Some(ls));
    let ob2 = Object::new(&sp, &sf1, &None);
    assert_eq!(ob2.light, None);

    // emittance
    let sp = Ray::new(&Vector3::O, &Vector3::EY).unwrap();
    let vvec = Vector3::new(1.0, -1.0, 0.0).normalize().unwrap();
    assert_eq!(ob1.emittance(&sp, &(-Vector3::EY)), ls.emittance(&sp, &(-Vector3::EY)));
    assert_eq!(ob1.emittance(&sp, &vvec), ls.emittance(&sp, &vvec));
    assert_eq!(ob2.emittance(&sp, &vvec), Radiance::ZERO);

    // flux
    assert_eq!(ob1.light.unwrap().radiosity(), 2.1961932650073206);  // [W]
    let fl = 2.1961932650073206 * PI4 * 0.5 * 0.5;
    assert_eq!(fl, 6.89954462721038);
    assert_eq!(ob1.flux(), fl);
    assert_eq!(ob2.flux(), 0.0);

    // emit_photons
    let ps = ob1.emit_photons(0.6899);
    assert_eq!(ps.len(), 10);

  }
}


// SUPPORT FUNCS




