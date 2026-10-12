// material

use std::fmt;

use super::{Flt};
use super::algebra::{BasicMatrix, Vector, Vector2};
use super::optics::{Color, Wavelength, approx_reflectance};
//use super::surface::{};
use super::utils::{russian_roulette_binary};

// PhotonBehavior -------------
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PhotonBehavior {
  Absorption,            // 吸収
  DiffuseReflection,     // 拡散反射
  SpecularReflection,    // 鏡面反射
  SpecularTransmission,  // 鏡面透過
}

// Material -------------------
//
//   物質の屈折率一覧： https://www.kla.com/ja/products/instruments/refractive-index-database
//                   https://refractiveindex.info/?shelf=main&book=Au&page=Johnson

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material {
  albedo_diff  : Color,  // アルベド（拡散）
  albedo_spec  : Color,  // アルベド（鏡面）-> 金属は屈折率と吸光係数(extinction coefficient)から計算、それ以外は屈折率から計算する
  transmittance: Color,  // 透過率
  ior          : Color,  // 屈折率(index of refraction) ※正確には真空に対する比屈折率
  scatterance  : Flt,    // 散乱度
  metalness    : Flt,    // 金属性 (0.0 〜 1.0:完全な金属)
}

impl Material {
  pub const AIR: Material = Material {albedo_diff: Color::WHITE, albedo_spec: Color::BLACK, transmittance: Color::WHITE,
                                      ior: Color::WHITE, scatterance: 0.0, metalness: 0.0 };


  #[inline(always)]
  pub fn new(aldiff: &Color, tran: &Color, ior: &Color, kappa: &Color, scatterance: Flt, metalness: Flt) -> Self {
    // アルベド(鏡面)は複素屈折率（ior, kappa）から計算で求める
    let albedo_spec = Color(
      calc_reflectance(ior.0, kappa.0),
      calc_reflectance(ior.1, kappa.1),
      calc_reflectance(ior.2, kappa.2)
    );
    Material{albedo_diff: *aldiff, albedo_spec, transmittance: *tran, ior: *ior, scatterance, metalness}
  }

  #[inline(always)]
  pub fn average_ior(&self) -> Flt {
    (self.ior.0 + self.ior.1 + self.ior.2) / 3.0
  }

  // is_specular
  //   鏡面反射するか？
  #[inline(always)]
  pub fn is_specular(&self) -> bool {
    self.metalness == 1.0 || self.albedo_spec != Color::BLACK  // 金属もしくは鏡面反射成分があれば真
  }

  // is_transparent
  //   透過するか？
  pub fn is_transparent(&self) -> bool {
    if self.metalness != 1.0 {
      // 完全な金属ではない
      if self.scatterance < 1.0 && self.albedo_diff != Color::BLACK {
        true
      } else {
        // 真の黒もしくは完全散乱する物質は透過しない
        false
      }
    } else {
      // 完全な金属は透過しない
      false
    }
  }

  // is_scatterred
  #[inline(always)]
  pub fn is_scatterred(&self) -> bool {
    self.scatterance != 0.0
  }

  // photon_behavior
  pub fn photon_behavior(&self, cos: Flt, wl: Wavelength) -> PhotonBehavior {
    if russian_roulette_binary(approx_reflectance(self.albedo_spec.wavelength(wl), cos)) { // 反射率以下なら鏡面反射
      PhotonBehavior::SpecularReflection
    } else {
      if !russian_roulette_binary(self.albedo_diff.wavelength(wl)) { // 拡散反射率を超えたら吸収
        PhotonBehavior::Absorption
      } else {
        if russian_roulette_binary(self.scatterance) { // 散乱度以下なら拡散反射
          PhotonBehavior::DiffuseReflection
        } else {
          PhotonBehavior::SpecularTransmission
        }
      }
    }
  }

  // store_photon
  //   フォトンマップに記録するか否か
  pub fn store_photon(&self) -> bool {
    self.metalness != 1.0 && self.scatterance != 0.0
  }

}

impl fmt::Display for Material {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "MATERIAL[{},{},{},{},{:.3},{:.3}]",
           self.albedo_diff, self.albedo_spec, self.transmittance, self.ior, self.scatterance, self.metalness)
  }
}

#[test]
fn test_material() {
  // new
  let m1 = Material::new(&Color::WHITE, &Color::BLACK,
                                   &Color(0.13, 0.45, 1.43), &Color(4.06, 2.42, 1.95), 0.0, 1.0);
  assert_eq!(m1.albedo_diff, Color::WHITE);
  assert_eq!(m1.albedo_spec, Color(0.9707215450015482, 0.7738380932038346, 0.41075880256299324));
  assert_eq!(m1.transmittance, Color::BLACK);
  assert_eq!(m1.ior, Color(0.13, 0.45, 1.43));
  assert_eq!(m1.scatterance, 0.0);
  assert_eq!(m1.metalness, 1.0);
  let m2 = Material::new(&Color::WHITE, &Color::BLACK,
                                   &Color::WHITE, &Color::BLACK, 0.0, 0.0);
  assert_eq!(m2.albedo_spec, Color::BLACK);
  assert_eq!(m2.metalness, 0.0);
  // average_ior
  assert_eq!(m1.average_ior(), (0.13 + 0.45 + 1.43) / 3.0);
  // is_specular
  assert_eq!(m1.is_specular(), true);
  assert_eq!(m2.is_specular(), false);
  // is_transparent
  assert_eq!(m1.is_transparent(), false);
  assert_eq!(m2.is_transparent(), true);
  let m3 = Material::new(&Color::WHITE, &Color::BLACK,
                                   &Color::WHITE, &Color::BLACK, 1.0, 0.0);
  assert_eq!(m3.is_transparent(), false);
  let m4 = Material::new(&Color::BLACK, &Color::BLACK,
                                   &Color::WHITE, &Color::BLACK, 0.0, 0.0);
  assert_eq!(m3.is_transparent(), false);
  // is_scatterred
  assert_eq!(m1.is_scatterred(), false);
  assert_eq!(m3.is_scatterred(), true);
  // photon_behavior
  assert_eq!(m1.photon_behavior(0.0, Wavelength::Red), PhotonBehavior::SpecularReflection);
  assert_eq!(m2.photon_behavior(1.0, Wavelength::Red), PhotonBehavior::SpecularTransmission);
  assert_eq!(m3.photon_behavior(1.0, Wavelength::Red), PhotonBehavior::DiffuseReflection);
  assert_eq!(m4.photon_behavior(1.0, Wavelength::Red), PhotonBehavior::Absorption);
  // store_photon
  assert_eq!(m1.store_photon(), false);
  assert_eq!(m2.store_photon(), false);
  assert_eq!(m3.store_photon(), true);
  assert_eq!(m4.store_photon(), false);

  // fmt
  assert_eq!(format!("{}", m1), "MATERIAL[COL[1,1,1],COL[0.9707215450015482,0.7738380932038346,0.41075880256299324],COL[0,0,0],COL[0.13,0.45,1.43],0.000,1.000]");



}


// SUPPORT FUNCS

// _calc_reflectance: 特定波長の真空に対する非屈折率（複素数 RE + iIM）から反射率と屈折率を求める
//   反射率の求め方
//   入射角ゼロにおける反射率(F0)は真空に対する比屈折率(η)から計算する
//   　　F0 = {(1 - η)/(1 + η)}^2
//        ※ 複素数でも同様
//          f = {(1 - (RE + iIM))/(1 + (RE + iIM))}^2
//          F0 = |f| ※ fは複素数なのでnormを使う
//   IN : 非屈折率(複素数)
//   OUT: 反射率(入射角ゼロ)

fn calc_reflectance(re: Flt, im: Flt) -> Flt {
  if im == 0.0 {
    // 金属以外は実部だけで計算
    let n = (1.0 - re) / (1.0 + re);
    return n * n
  }

  let n_nu = Vector2::new(1.0 - re, -im);
  let n_de = Vector2::new(1.0 + re, im);
  let n_de_ = Vector2::new(n_de.x, -n_de.y);

  let deno = n_de.square();
  let nume = Vector2::new(n_nu.x * n_de.x - n_de.y * n_de.y, -(n_de_.x * n_nu.y + n_nu.x * n_de_.y)) / deno;
  match nume {
    None => 0.0,
    Some(f_) => Vector2::new(f_.x * f_.x - f_.y * f_.y, 2.0 * f_.x * f_.y).norm(),
  }

}

#[test]
fn test_support_material() {
  // calc_reflectance
  assert_eq!(calc_reflectance(1.0, 0.0), 0.0);
  assert_eq!(calc_reflectance(1.333, 0.0), 0.020373187841971414);  // 水
  assert_eq!(calc_reflectance(1.5, 0.0),  0.04000000000000001);    // ガラス

  // 金（700.0, 546.1, 435.5)
  assert_eq!(calc_reflectance(0.131, 4.062), 0.9705270345556459);
  assert_eq!(calc_reflectance(0.447, 2.421), 0.775237113531656);
  assert_eq!(calc_reflectance(1.433, 1.949), 0.4101721634601038);
  // 金（610.0, 550.0, 546.1, 465.0)
  assert_eq!(calc_reflectance(0.226, 3.192), 0.9226817790717367);
  assert_eq!(calc_reflectance(0.424, 2.472), 0.7916093264656153);
  assert_eq!(calc_reflectance(1.332, 1.869), 0.4034519842107356);


}

