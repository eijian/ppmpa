// camera module

use rand::Rng;
use std::collections::HashMap;


use crate::ray::{Flt, PI2};

use crate::ray::algebra::{Direction3, Position3, Ray, Vector, Vector2, Vector3};
use crate::ray::optics::{PhotonFilter, Radiance};

use super::parser::*;


// CONSTANTS & DEFAULTS
//

// γ補正値
pub const GAMMA: Flt = 1.0 / 2.2;

// 標準EV値=14とする（晴天時の屋外相当）
pub const STANDARD_EV: Flt = 14.0;

// 標準ISO感度=100とする
pub const STANDARD_ISO: Flt = 100.0;

// 撮像素子サイズ 36mm x 24mmとし、横浜36mmを基準とする
pub const SENSOR_SIZE : Flt = 36.0 / 1000.0;

// 標準ホワイトバランス=5200K(CANON一眼の"太陽光")
// cf. https://faq.canon.jp/app/answers/detail/a_id/61330/~/%E3%80%90%E3%83%87%E3%82%B8%E3%82%BF%E3%83%AB%E4%B8%80%E7%9C%BC%E3%83%AC%E3%83%95%E3%82%AB%E3%83%A1%E3%83%A9%E3%83%BB%E3%83%9F%E3%83%A9%E3%83%BC%E3%83%AC%E3%82%B9%E3%82%AB%E3%83%A1%E3%83%A9%E3%80%91%E3%83%9B%E3%83%AF%E3%82%A4%E3%83%88%E3%83%90%E3%83%A9%E3%83%B3%E3%82%B9%E3%81%A8%E3%81%AF%EF%BC%9F)
pub const STANDARD_WB : Flt = 5200.0;

// カメラパラメータ（デフォルト）

pub const CAMERA_DEFAULT: [(&str, &str); 13] = [
  (FOCAL_LENGTH   , "50"),      // 焦点距離 50mm
  (F_NUMBER       , "100"),         // F値 100 = pin hole camera
  (SHUTTER_SPEED  , "50"),     // シャッタースピード 1/50
  (ISO_SENSITIVITY, "100"),  // ISO感度 100
  (EV_ADJUSTMENT  , "0.0"),
  (WHITE_BALANCE  , "5200"),
  (FOCUS_DISTANCE , "2.0"),
  (RESOLUTION     , "(256, 256)"),
  (ANTIALIAS      , "True"),
  (PHOTON_FILTER  , "Nonfilter"),
  (EYE_POSITION   , "Vector3 0.0 2.0 -4.5"),
  (TARGET_POSITION, ""),
  (UPPER_DIRECTION, ""),
];

// Camera ----------------

pub struct Camera {
  // camera specs
  pub focal_length  : Flt,  // 焦点距離(mm)
  pub fnumber       : Flt,  // F値
  pub shutter_speed : Flt,  // シャッタースピード(/sec)
  pub white_balance : Flt,  // ホワイトバランス(K) <- 標準値をCANONカメラ太陽光(5200K)とした時の差
  pub focus_distance: Flt,  // フォーカス距離(m)
  pub exposure      : Flt,  // 光量(露出)
  pub lens          : (Direction3, Direction3),  // レンズXYベクトル
  pub blur_flag     : bool,                      // ボケ表現ON/OFF
  
  // screen quality
  pub resolution    : (i16, i16),                // 画面解像度
  pub antialias     : bool,                      // アンチエリアスON/OFF
  //pub nphoton       : Int,                       // フォトン数
  //pub radius        : Flt,                       // フォトン収集半径
  pub pfilter       : PhotonFilter,              // 収集時フィルター
  
  // visual environment
  pub eye_pos       : Position3,                 // 視点
  pub eye_dir       : Direction3,                // 視線
  pub screen_origin : Position3,                 // スクリーン開始点
  pub screen_step   : (Direction3, Direction3),  // スクリーン移動量
  //pub screen_map    : Vec<(Flt, Flt)>,           // スクリーン位置集合

  // for output
  //pub pnm_header    : Vec<Char>,                 // PNMファイルヘッダ
  //pub ambient       : Radiance,                  // 環境光輝度
}

impl Camera {
  pub fn new(_param: &HashMap<&str, &str>) -> Camera {
    let focallen0 = 30.0;
    let fnumber = 4.0;
    let shutterspd = 1000.0;
    let isosens = 100.0;
    let evadjust = 0.0;
    let wb0 = 5200.0;
    let focusdist = 3.0;
    let reso = (320, 240);
    let antialias = true;
    //let nphoton = 100000;
    //let radius = 0.1;
    let pfilt = PhotonFilter::Conefilter;
    let eyepos = Position3::new(0.0, 2.0, -2.5);
    let targetpos = Position3::new(0.0, 2.0, 0.0);
    let upper = Vector3::EY;
    //let amb = Radiance(0.001, 0.001, 0.001);


    let focallen = focallen0 / 1000.0;   // 焦点距離[mm]を[m]単位に変換

    let ez = (targetpos - eyepos).normalize().unwrap();
    let ex = upper * ez;
    let ey = ex * ez;

    // 露出(exposure)は得られた輝度を何倍するかを表しており、露出係数EVとISO感度から求める。
    // 露出の計算には、2の(露出係数は標準EV値との差)乗倍 × 標準ISO感度倍率(=ISO感度) を用いる
    // 
    // 露出係数EVの決め方
    // 参考: https://www.ccs-inc.co.jp/guide/column/light_color/vol26.html
    //   EV = AV + TV <- 小さいほど明るい
    //     where AV=log_2 A^2 (A=F値)
    //           TV=log_2 (1/T) (T=露光時間(s)=1/シャッタースピード(/s))
    //   ev = (logBase 2 (fnumber ** 2)) + (logBase 2 shutterspd) - evadjust
    let ev = f64::log2(fnumber * fnumber) + f64::log2(shutterspd) - evadjust;
    let exposure = 2.0_f64.powf(STANDARD_EV - ev) * (isosens / STANDARD_ISO);

    let wb = STANDARD_WB - wb0;  // 基準ホワイトバランスとの差分を保持

    let ea = focallen / fnumber / 2.0;   // 開放口径（の半径）
    let lens = (ea * ex, ea * ey);
    let blurflag = fnumber < 100.0;  // F値が100未満なら通常、100以上ならピンホールカメラとする（＝ブレを作らない）

    //let radius2 = radius * radius;
    let eyedir = ez;

    let (xr, yr) = reso;
    let step = (focusdist * SENSOR_SIZE / focallen) / (xr as Flt);
    let sstep = (step * ex, step * ey);
    let hr_x = (xr / 2) as Flt;  // half resolution
    let hr_y = (yr / 2) as Flt;
    let orig = eyepos + focusdist * ez - (hr_x * sstep.0) + (hr_y * sstep.1);  // 仮想スクリーンの左上ポイント

    Camera {
      focal_length  : focallen,          // 焦点距離[m]
      fnumber       : fnumber,           // F値
      shutter_speed : shutterspd,        // シャッタースピード[/s]
      white_balance : wb,                // ホワイトバランス
      focus_distance: focusdist,         // 
      exposure      : exposure,          // 露出
      lens          : lens,              // レンズXYベクトル
      blur_flag     : blurflag,          // ボケ表現 ON/OFF
      resolution    : reso,              // 画面解像度
      antialias     : antialias,         // アンチエリアス ON/OFF
      pfilter       : pfilt,             // 収集時フィルター
      eye_pos       : eyepos,            // 視点
      eye_dir       : eyedir,            // 視線
      screen_origin : orig,              // スクリーン開始点 (左上をx,y=0,0とし、そのピクセルの左上の座標とする)
      screen_step   : sstep,             // スクリーン移動量
    }

  }

  // 露出補正
  #[inline(always)]
  pub fn compensate_exposure(&self, rad: &Radiance) -> Radiance {
    self.exposure * *rad
  }

  // 視点からの光線を生成
  pub fn generate_ray(&self, pos: &Vector2) -> Ray {
    let pixel = Vector2::new(f64::min(f64::max(0.0, pos.x), self.resolution.0 as Flt - 1.0),
                                      f64::min(f64::max(0.0, pos.y), self.resolution.1 as Flt - 1.0));


    // 仮想スクリーンの左上を起点に、pixelでスクリーン上を移動し、offset分ピクセル内にずれた位置を視線の向かう先とする。
    // レンズ上のランダムな点からその位置までを視線ベクトルとする。
    let offset = Camera::_pixel_offset(self.antialias);
    let blur = Camera::_lens_offset(self.blur_flag, self.lens);
    let eyepos = self.eye_pos + blur;
    let eyedir = self.screen_origin + (pixel.x + offset.x) * self.screen_step.0 - (pixel.y + offset.y) * self.screen_step.1 - eyepos;
    Ray::new(&eyepos, &eyedir).unwrap()
  }

  //  ピクセル内のランダムな点を選ぶためにズレを求める
  fn _pixel_offset(antialias: bool) -> Vector2 {
    if antialias {
      let mut rng = rand::thread_rng();
      let off_x = rng.gen_range(0.0, 1.0) - 0.5;
      let off_y = rng.gen_range(0.0, 1.0) - 0.5;
      Vector2::new(off_x, off_y)
    } else {
      Vector2::new(0.5, 0.5)
    }
  }

  fn _lens_offset(blurflag: bool, lens: (Position3, Position3)) -> Position3 {
    if blurflag == true {
      // レンズ上のランダムな点を生成する方法) 二次元極座標を使う
      // r1=半径(平方根を取ることで密度を一定にする)、r2=角度
      // https://techblog.kayac.com/how-to-distribute-points-randomly-using-high-school-math
      let mut rng = rand::thread_rng();
      let r1: Flt = f64::sqrt(rng.gen_range(0.0, 1.0));  // 径方向 (0 〜 1)
      let r2: Flt = rng.gen_range(0.0, PI2);                   // 円周方向（0 〜 2π)
      r1 * f64::cos(r2) * lens.0 + r1 * f64::sin(r2) * lens.1
    } else {
      Vector3::O
    }
  }

}

#[test]
fn test_camera() {

}

