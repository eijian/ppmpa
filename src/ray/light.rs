// light

//use std::f64;
use rand::Rng;

use super::{Flt, NEARLY0, PI, SR_HALF};
use super::algebra::{BasicMatrix, Direction3, Ray, SurfacePoint, Vector, Vector3, blurred_vector};
use super::optics::{Color, Photon, Radiance, Wavelength};
use super::utils::{density_power};

// Flux: 放射束 (W)
pub type Flux = Flt;

// Radiosity: 放射発散度,ラジオシティ [W/m^2]
pub type Radiosity = Flt;

// RadEstimation: 直接光の放射輝度推定方法
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RadEstimation {
  Formula,  // 古典レイトレーシング(計算)で求める
  Photon,   // フォトンマップから輝度推定
}

// InOut (In or Out)
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InOut {
  In,
  Out,
}

// LightSpec ----------------------
/*
  光源型、物体形状と分離して光源の仕様のみとした。

  参考光源情報）

  Panasonic (https://panasonic.jp/lamp/)
  電球型(LDA13(L/N/D)-G/Z100E/S/W, LDA7(L/WW/N/D)-D-G/S/Z6)
    電球色   温白色   昼白色   昼光色
    2700K   3500K   5000K   6500K 

    100W     60W     40W
    1520lm   810lm   485lm
  
  丸型(FCL30EL28MF2)
    電球色   ナチュラル色  クール色
    3000K    5200K      6200K

    30W     32W     40W
    2100lm  2480lm  3230lm

  Panasonic 丸型LED 95mm
  LDG11L-G/95/W  1370lm -> 46,320 lx (lm/m^2)

  Panasonic 円環30形 (225-167,29mm)
  FCL30EDW28MF3  2100lm -> 74,868 lx (lm/m^2)
 */

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightSpec {
  color      : Color,         // RGBの比率、r+g+b = 1.0
  radiosity  : Radiosity,     // 放射発散度 [W/m^2]
  directivity: Flt,           // 指向性 (0.0:指向性なし 〜 1.0:平行光源)
  radest     : RadEstimation, // 直接光の輝度値計算方法
  dir_flag   : InOut,         // 発光方向（通常光源=Out, ドーム光源等=In)
  // calcuration when initializing
  cospower   : Flt,
  power      : Flt,
  emittance0 : Radiance,      // 放射輝度 (面法線方向)
}

impl LightSpec {

  #[inline(always)]
  pub fn new(color: &Color, lux: Radiosity, directivity: Flt, radest: RadEstimation, dir_flag: InOut) -> Self {
    let radiosity = lux / 683.0;  // flux [W] = lumen / 683.0, lux = lumen / S [lumen/m^2]
    let (cospower, power) = density_power(directivity.powf(3.0));
    let emittance0 = radiosity * *color * Radiance::ONE;

    LightSpec {color: *color, radiosity, directivity, radest, dir_flag, cospower, power, emittance0}
  }

  pub fn radiosity(&self) -> Radiosity {
    self.radiosity
  }

  // 光源表面の点spから視線方向(vvec)への放射輝度
  pub fn emittance(&self, sp: &SurfacePoint, vvec: &Direction3) -> Radiance {
    let cos = -sp.dir.dot(vvec);

    //cos.powf(0.5 / self.power) * self.emittance0
    self.decay_emittance(cos)
  }

  // 光源から放射される光子の生成
  pub fn generate_photon(&self, sp: &SurfacePoint) -> (Photon, RadEstimation) {
    let nvec_ = if self.dir_flag == InOut::Out {
      sp.dir
    } else {
      -sp.dir
    };
    let nvec2 = blurred_vector(&nvec_, self.power);
    let mut rng = rand::thread_rng();
    let wl = self.color.decide_wavelength(rng.gen_range(0.0, 1.0));

    (Photon(wl, Ray::new(&sp.pos, &nvec2).unwrap()), self.radest)
  }

  // 光源表面から拡散する直接光の減衰量を計算で求める。
  // 光源から放射される光強度は発散地点の法線ベクトルに対する放射角に依存する（余弦則）が、
  // 余弦のn乗に従う場合は角度による減衰に1/2π^(1/n+1)を掛けたものと一致する。
  // 1/2π^(1/n+1) cos^n θ
  // 例) n=0のとき、半球に均一に発散、強度は 1/2π倍
  //     n=♾️のとき、面法線方向にのみ発散、強度は 1倍
  pub fn decay_emittance(&self, cos: Flt) -> Radiance {
    self._decay(cos) * self.emittance0
  }

  // 光源までの距離は二乗された状態で入力される。1/4π・d となっているがdは実際はd^2。
  pub fn get_radiance(&self, lnvec: &Direction3, lvec: &Direction3) -> Radiance {
    let len2 = lvec.square();
    if len2 == 0.0 {
      Radiance::ZERO // 照射点が光源と接している（エラー）
    } else {
      let len = f64::sqrt(len2);
      let cos = lvec.dot(lnvec) / len;
      if cos < 0.0 {
        // 照射点が光源の発散方向の裏側
        Radiance::ZERO
      } else {
        (1.0 / len2) * self.decay_emittance(cos) 
      }
      //Radiance::from_color(&self.color, SR_HALF / (len * len) * self.radiosity * self._decay(cos))
    }
  }

  fn _decay(&self, cos: Flt) -> Flt {
    //cos.powf(self.cospower) / (self.cospower + 1.0) // 減衰するためには割り算？
    cos.powf(self.cospower) * SR_HALF.powf(self.power)
  }

}

#[test]
fn test_lightspec() {
  // new
  let col = Color::new_by_kelvin(5000.0);
  let ls = LightSpec::new(&col, 1500.0, 0.0, RadEstimation::Photon, InOut::Out);
  assert_eq!(ls.color, Color(0.3701322600524373, 0.3309599290794973, 0.29890781086806545));
  assert_eq!(ls.radiosity, 1500.0 / 683.0);
  assert_eq!(ls.directivity, 0.0);
  assert_eq!(ls.radest, RadEstimation::Photon);
  assert_eq!(ls.cospower, 1.0);
  assert_eq!(ls.power, 0.5);
  let em = 1500.0 / 683.0;
  let r0 = Radiance(em * 0.3701322600524373, em * 0.3309599290794973, em * 0.29890781086806545);
  // r0 = Radiance(0.12937418474038126, 0.11568208348099218, 0.1044787458896705)
  assert_eq!(ls.emittance0, r0);
  let ls1 = LightSpec::new(&col, 1500.0, 1.0, RadEstimation::Formula, InOut::Out);
  assert_eq!(ls1.directivity, 1.0);
  assert_eq!(ls1.radest, RadEstimation::Formula);
  assert_eq!(ls1.cospower, (10.0 as f64).powf(10.0));
  assert!(ls1.power < NEARLY0);
  let ls2 = LightSpec::new(&col, 1500.0, -1.0, RadEstimation::Formula, InOut::Out);
  assert_eq!(ls2.directivity, -1.0);
  assert_eq!(ls2.radest, RadEstimation::Formula);
  assert!(ls2.cospower < NEARLY0);
  assert!(1.0 - ls2.power < NEARLY0);



  // lemittance
  let sp = Ray::new(&Vector3::O, &Vector3::EY).unwrap();
  let vvec = Vector3::new(1.0, -1.0, 0.0).normalize().unwrap();
  let cos = f64::cos(PI / 4.0);
  assert_eq!(ls.emittance0, r0);
  let r1 = SR_HALF.powf(0.5) * cos * ls.emittance0;
  assert!(ls.emittance(&sp, &vvec).near(&r1));

  // generate_photon
  let ls3 = LightSpec::new(&col, 1500.0, 1.0, RadEstimation::Photon, InOut::Out);
  let sp = Ray::new(&Vector3::O, &Vector3::EY).unwrap();
  let p1 = Photon::new(Wavelength::Red, &sp);
  let (p2, re) = ls3.generate_photon(&sp);
  assert!(p2.1.dir.near(&p1.1.dir));
  assert_eq!(re, RadEstimation::Photon);


  // decay_emittance
  //   - cos^1の余弦則
  let dec = SR_HALF.powf(1.0 / 2.0);
  assert_eq!(ls.decay_emittance(1.0), dec * ls.emittance0);                // 0度: 1/2pi^0.5
  let rt22 = 1.0 / f64::sqrt(2.0);
  let cos4 = f64::cos(PI / 4.0);
  assert!(ls.decay_emittance(cos4).near(&(dec * rt22 * ls.emittance0)));   // 45度: 1/2pi^0.5 * cos
  assert_eq!(ls.decay_emittance(0.0), Radiance::ZERO);                     // 90度  <- cos = 0.0
  //   - cos^♾️の余弦則
  assert!(ls1.decay_emittance(1.0).near(&ls1.emittance0));                 // 0度: 1/2pi^0 = 1
  //assert_eq!(ls1.decay_emittance(0.0), Radiance::ZERO);
  assert_eq!(ls1.decay_emittance(cos4), Radiance::ZERO);                   // 45度: 1/2pi^0 * cos pi/4 ^無限大 -> 0
  assert_eq!(ls1.decay_emittance(0.0), Radiance::ZERO);                    // 90度: 1/2pi^0 * cos pi/4 ^無限大 -> 0
  //   - cos^0の余弦則
  assert!(ls2.decay_emittance(1.0).near(&(SR_HALF * ls2.emittance0)));     // 0度: 1/2pi^1
  assert!(ls2.decay_emittance(cos4).near(&(SR_HALF * ls2.emittance0)));    // 45度: 1/2pi^1 * cos^0 -> 1/2pi
  assert!(ls2.decay_emittance(0.0).near(&(SR_HALF * ls2.emittance0)));     // 45度: 1/2pi^1 * cos^0 -> 1/2pi

  // get_radiance  
  //   - cos^1の余弦則
  let mut len = 3.0 * 3.0;
  assert_eq!(ls.get_radiance(&Vector3::EY, &(3.0 * Vector3::EY)), 1.0 / len * ls.decay_emittance(1.0));
  assert_eq!(ls.get_radiance(&Vector3::EY, &(Vector3::O)), Radiance::ZERO);
  let lv = Vector3::new(-2.0, 2.0, 0.0);
  len = lv.square();
  let cos = 1.0 / f64::sqrt(2.0);
  assert_eq!(ls.get_radiance(&Vector3::EY, &lv), 1.0 / len * ls.decay_emittance(cos));
  //     裏側だったら輝度ゼロ
  assert_eq!(ls.get_radiance(&Vector3::EY, &(-lv)), Radiance::ZERO);

}


