// surface
//   Reference URLs
//   https://www.slideshare.net/teppeikurita/brdf-196782059
//

use std::fmt;
//use rand::Rng;

use super::{Flt, NEARLY0, PI};
use super::algebra::{Direction3, SurfacePoint, Vector, Vector2, Vector3, blurred_vector};
use super::material::{Material};
use super::optics::{Color};
use super::utils::{density_power, russian_roulette_binary};

// Mapping ---------------
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mapping<'a> {
  Solid {
    mate: &'a Material,
  },
  Checker {
    mate1: &'a Material,
    mate2: &'a Material,
    mag  : Flt,
  },
}

impl<'a> Mapping<'a> {
  #[inline(always)]
  pub fn new_solid(mate: &Material) -> Mapping {
    Mapping::Solid {mate}
  }

  #[inline(always)]
  pub fn new_checker(mate1: &'a Material, mate2: &'a Material, mag: Flt) -> Mapping<'a> {
    Mapping::Checker {mate1, mate2, mag}
  }

  pub fn material(&self, sp: &SurfacePoint, _uv: &Vector2) -> &Material {
    match self {
      Mapping::Solid {mate}
        => mate,
      Mapping::Checker {mate1, mate2, mag}
        => if f64::sin(sp.pos.x * PI * mag) * f64::sin(sp.pos.z * PI * mag) > 0.0 {
          mate1
        } else {
          mate2
        }
    }
  }

}

impl<'a> fmt::Display for Mapping<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Mapping::Solid {mate}
        => write!(f, "SOLID[{}]", mate),
      Mapping::Checker {mate1, mate2, mag}
        => write!(f, "CHECKER[{},{},{:.3}]", mate1, mate2, mag),
    }
  }
}

#[test]
fn test_mapping() {
  // new
  let m1 = Material::new(&Color::WHITE, &Color::BLACK,
                                   &Color(0.13, 0.45, 1.43), &Color(4.06, 2.42, 1.95), 0.0, 1.0);
  let mp1 = Mapping::new_solid(&m1);
  let sp1 = SurfacePoint::new(&Vector3::new(0.5, 0.0, 0.5), &Vector3::EY).unwrap();
  let uv = Vector2::O;
  assert_eq!(*mp1.material(&sp1, &uv), m1);
  let m2 = Material::new(&Color::WHITE, &Color::BLACK,
                                   &Color::WHITE, &Color::BLACK, 0.0, 0.0);
  let sp2 = SurfacePoint::new(&Vector3::new(1.5, 0.0, 0.5), &Vector3::EY).unwrap();
  let mp2 = Mapping::new_checker(&m1, &m2, 1.0);
  assert_eq!(*mp2.material(&sp1, &uv), m1);
  assert_eq!(*mp2.material(&sp2, &uv), m2);
  let sp3 = SurfacePoint::new(&Vector3::new(0.5, 0.0, 1.5), &Vector3::EY).unwrap();
  assert_eq!(*mp2.material(&sp3, &uv), m2);
  
  // fmt
  assert_eq!(format!("{}", mp1), "SOLID[MATERIAL[COL[1,1,1],COL[0.9707215450015482,0.7738380932038346,0.41075880256299324],COL[0,0,0],COL[0.13,0.45,1.43],0.000,1.000]]");
  assert_eq!(format!("{}", mp2), "CHECKER[MATERIAL[COL[1,1,1],COL[0.9707215450015482,0.7738380932038346,0.41075880256299324],COL[0,0,0],COL[0.13,0.45,1.43],0.000,1.000],MATERIAL[COL[1,1,1],COL[0,0,0],COL[0,0,0],COL[1,1,1],0.000,0.000],1.000]");

}


// Surface --------------------------------
/*
  glossyな表面とは法線ベクトルがブレていると捉える。
  ブレ幅は表面の粗さ(roughness)に依る。
    完全に粗い  ：roughness=1.0
    完全に滑らか：roughness=0.0
  
  面の法線をN、glossy面でブレた法線をN'とすると
  <N',N'>(NとN'のなす角をθとした時のcosθ)は、乱数ξ[0,1]を用いて
    cosθ = ξ^(1/n+1)
  とする。ここでnは
    n = 10^(5*(1-√roughness))
  と決める。（roughness 0 〜 1の時の増分に結果がスライドするように）
  cosθの計算時に使えるよう保持しておく
  densityPow <- 1/(n+1)
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Surface<'a> {
  roughness: Flt,                  // 表面の粗さ(0.0: 完全平滑面 〜 1.0: 完全拡散面)
  pub map      : Mapping<'a>,          // 材質情報
  // calcuration when initializing
  pub power    : Flt,                  // 表面の粗さから分散度合いnを求める
  alpha    : Flt,                  // 粗さを４乗して保持しておく
}

impl<'a> Surface<'a> {

  #[inline(always)]
  pub fn new(rough: Flt, map: Mapping<'a>) -> Self {
    let (_, power) = density_power(1.0 - f64::sqrt(rough));
    let alpha = rough * rough * rough * rough;

    Surface {roughness: rough, map, power, alpha}
  }

  // microfacetNormal: 微小平面での法線ベクトルを求める
  /*
    IN:  self(surf)  表面状態
         nvec  マクロ平面での法線ベクトル
         vvec  光子/視線の交点への入射ベクトル
         retry 算出した法線ベクトルが無効な場合に再算出するかどうか[0:1回のみ試行 〜 1:有効な解が得られるまで試行]
    OUT: 微小平面での法線ベクトル（Maybe）
   */
  pub fn microfacet_normal(&self, nvec: &Direction3, vvec: &Direction3, retry: Flt) -> Option<Direction3> {
    if self.roughness == 0.0 {
      // 完全平滑面ならマクロ法線を返す
      Some(*nvec)
    } else {
      let nvec_ = blurred_vector(nvec, self.power);
      if nvec_.dot(vvec) < 0.0 {
        // 有効な法線ベクトルが得られた
        Some(nvec_)
      } else {
        // 金属なら法線ベクトルを必ず求める（retry = 1.0)
        if russian_roulette_binary(retry) {
          self.microfacet_normal(nvec, vvec, retry)
        } else {
          None
        }
      }
    }
  }

  pub fn material(&self, sp: &SurfacePoint, uv: &Vector2) -> &Material {
    self.map.material(sp, uv)
  }

}

impl<'a> fmt::Display for Surface<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Surface ({}, {})", self.roughness, self.power)
  }
}


#[test]
fn test_surface() {
  // new
  let m1 = Material::new(&Color::WHITE, &Color::BLACK,
                                   &Color(0.13, 0.45, 1.43), &Color(4.06, 2.42, 1.95), 0.0, 1.0);
  let mt1 = Mapping::new_solid(&m1);
  let sf1 = Surface::new(0.0, mt1);
  assert_eq!(sf1.roughness, 0.0);
  assert!(sf1.power < NEARLY0);
  assert_eq!(sf1.alpha, 0.0);

  // microfacet_normal
  assert_eq!(sf1.microfacet_normal(&Vector3::EY, &(-Vector3::EY), 1.0), Some(Vector3::EY));
  let vvec = Vector3::new(1.0, -1.0, 0.0).normalize().unwrap();
  assert_eq!(sf1.microfacet_normal(&Vector3::EY, &vvec, 1.0), Some(Vector3::EY));
  let m2 = Material::new(&Color::WHITE, &Color::BLACK,
                                   &Color::WHITE, &Color::BLACK, 1.0, 0.0);
  let mt2 = Mapping::new_checker(&m1, &m2, 1.0);
  let sf2 = Surface::new( 0.5, mt2);
  assert_eq!(sf2.roughness, 0.5);
  assert!(sf2.power - 0.0011 < NEARLY0);
  assert_eq!(sf2.alpha, (0.5_f64).powf(4.0));
  let rvec = sf2.microfacet_normal(&Vector3::EY, &(-Vector3::EY), 1.0);
  match rvec {
    Some(rv) => assert!(rv.dot(&Vector3::EY) >= 0.0),
    None => assert!(false),
  }

  // material
  let mut sp = SurfacePoint::new(&Vector3::new(0.5, 0.0, 0.5), &Vector3::EY).unwrap();
  assert_eq!(*sf1.material(&sp, &Vector2::O), m1);
  assert_eq!(*sf2.material(&sp, &Vector2::O), m1);
  sp = SurfacePoint::new(&Vector3::new(1.5, 0.0, 0.5), &Vector3::EY).unwrap();
  assert_eq!(*sf2.material(&sp, &Vector2::O), m2);

}


