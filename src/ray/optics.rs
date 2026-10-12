/// Optics

use core::num::ParseFloatError;
//use std::f64;
use std::fmt;
//use std::ops::ControlFlow::Break;
use std::ops::Neg;
use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;
use std::str::{FromStr};

use regex::Regex;

use kd_tree::{KdPoint, KdTree};

use super::{Flt, NEARLY0, PI};
use super::algebra::{BasicMatrix, Direction3, Position3, Ray, Vector, Vector3, generate_random_dir4};

// Wavelength (波長) -----------------------
//          CIE        Monitor
//   Red  : 700.0 nm   610.0 nm
//   Green: 546.1 nm   550.0 nm
//   Blue : 435.8 nm   465.0 nm

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Wavelength {
  Red,
  Green,
  Blue
}

impl fmt::Display for Wavelength {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let swl = match self {
      Wavelength::Red   => "Red",
      Wavelength::Green => "Green",
      Wavelength::Blue  => "Blue",
    };
    write!(f, "WL:{}", swl)
  }
}

impl FromStr for Wavelength {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let re = Regex::new(r"^WL:(\S+?)$").unwrap();
    let caps = re.captures(s).unwrap();
    match &caps[1] {
      "Red"   => Ok(Wavelength::Red),
      "Green" => Ok(Wavelength::Green),
      "Blue"  => Ok(Wavelength::Blue),
      _       => Err(format!("invalid wavelength: {}", s)),
    }
  }
}

#[test]
fn test_wl() {
  // test1
  let r = Wavelength::Red;
  assert_eq!(r, Wavelength::Red);
  assert_eq!(format!("{}", Wavelength::Red), "WL:Red");
  assert_eq!(format!("{}", Wavelength::Green), "WL:Green");
  assert_eq!(format!("{}", Wavelength::Blue), "WL:Blue");
  let sr = "WL:Red";
  assert_eq!(Wavelength::from_str(sr).unwrap(), Wavelength::Red);
  let sg = "WL:Green";
  assert_eq!(Wavelength::from_str(sg).unwrap(), Wavelength::Green);
  let sb = "WL:Blue";
  assert_eq!(Wavelength::from_str(sb).unwrap(), Wavelength::Blue);
  // test2
  let sx = "WL:aaa";
  let wl = Wavelength::from_str(sx);
  if let Err(e) = wl {
    assert_eq!(e, "invalid wavelength: WL:aaa")
  }
}

// Color -----------------------

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Color(pub Flt, pub Flt, pub Flt);

impl Color {
  pub const ZERO : Color = Color(0.0, 0.0, 0.0);
  pub const ONE  : Color = Color(1.0, 1.0, 1.0);
  pub const BLACK: Color = Color(0.0, 0.0, 0.0);
  pub const WHITE: Color = Color(1.0, 1.0, 1.0);
  pub const MIN:   Color = Color(1.0/10000.0, 1.0/10000.0, 1.0/10000.0);

  #[inline(always)]
  pub fn new(r: Flt, g: Flt, b: Flt) -> Color {
    // normalize() は明示的に実行する
    Color(r, g, b)
  }

  /*
    正規化と確率分布化

      正規化: normalize -> 数値群の最大値を1.0とし、そのほかを1.0との比率に変換する
        例): 2.4, 1.3, 0.5 => 1.0, 0.54, 0.21
      確率分布化: probabilize -> 数値群の合計が1.0となるように、各値を比率按分する
        例): 2.4, 1.3, 0.5 => 0.57, 0.31, 0.12
   */

  #[inline(always)]
  pub fn new_normalize(r: Flt, g: Flt, b: Flt) -> Color {
    Color(r, g, b).normalize()
  }

  #[inline(always)]
  pub fn new_probabilize(r: Flt, g: Flt, b: Flt) -> Color {
    Color(r, g, b).probabilize()
  }


  /*
    色温度(Kelvin) -> Color
    in : color temperature (K)
    out: Color
  */

  // 変換アルゴリズム1:
  // https://ja.visual-foxpro-programmer.com/color-temperature-rgb-conversion
  #[inline(always)]
  pub fn new_by_kelvin(t: Flt) -> Color {
    fn clip(c: Flt) -> Flt {
      if c > 255.0 {
        1.0
      } else {
        c / 255.0
      }
    }

    let t2 = t / 100.0;
    let r = if t2 <= 66.0 {
      255.0
    } else {
      329.698727446 * (t2 - 60.0).powf(-0.1332047592)
    };
    let g = if t2 <= 66.0 {
      //99.4708025861 * f64::log(f64::exp(1.0), t2) - 161.1195681661
      99.4708025861 * t2.ln() - 161.1195681661
    } else {
      288.1221695283 * (t2 - 60.0).powf(-0.0755148492)
    };
    let b = if t2 >= 66.0 {
      255.0
    } else {
      //138.5177312231 * f64::log(f64::exp(1.0), t2 - 10.0) - 305.0447927307
      138.5177312231 * (t2 - 10.0).ln() - 305.0447927307
    };

    Color(clip(r), clip(g), clip(b)).probabilize()
  }

  // 変換アルゴリズム2:
  // https://mahiroapps.com/color-temperature-calc
  #[inline(always)]
  pub fn new_by_kelvin2(t: Flt) -> Color {
    let x = if t <= 4000.0 {
      -0.2661239e9 / (t * t * t) - 0.2343589e6 / (t * t) + 0.8776956e3 / t + 0.179910
    } else {
      -3.0258469e9 / (t * t * t) + 2.1070379e6 / (t * t) + 0.2226347e3 / t + 0.240390
    };
    let y = if t <= 2222.0 {
      -1.1063814 * x * x * x - 1.34811020 * x * x + 2.18555832 * x - 0.20219683
    } else if t <= 4000.0 {
      -0.9549476 * x * x * x - 1.37418593 * x * x + 2.09137015 * x - 0.16748867
    } else {
       3.0817580 * x * x * x - 5.87338670 * x * x + 3.75112997 * x - 0.37001483
    };

    let x2 = x * 1.0 / y;
    let y2 = 1.0;
    let z2 = (1.0 - x - y) * 1.0 / y;

    fn clip(c: Flt) -> Flt {
      if c < 0.0 {
        0.0
      } else {
        c.powf(1.0 / 2.2)
      }
    }

    let r = clip( 3.2406 * x2 - 1.5372 * y2 - 0.4986 * z2);
    let g = clip(-0.9689 * x2 + 1.8758 * y2 + 0.0415 * z2);
    let b = clip( 0.0557 * x2 - 0.2040 * y2 + 1.0570 * z2);

    Color(r, g, b).probabilize()
  }

  // 変換テーブル:
  // https://mahiroapps.com/color-temperature-calc
  #[inline(always)]
  pub fn new_by_kelvin3(t: Flt) -> Color {
    let t2 = if t < 0.0 {
      0.0
    } else {
      if t > 12000.0 {
        12000.0
      } else {
        t
      }
    };
    Self::KELVIN_TABLE[(t2 / 100.0) as usize].probabilize()
  }

  const KELVIN_TABLE: [Color; 121] = [
    Color(1.000, 0.220, 0.000),
    Color(1.000, 0.220, 0.000),
    Color(1.000, 0.220, 0.000),
    Color(1.000, 0.220, 0.000),
    Color(1.000, 0.220, 0.000),
    Color(1.000, 0.220, 0.000),
    Color(1.000, 0.220, 0.000),
    Color(1.000, 0.220, 0.000),
    Color(1.000, 0.220, 0.000),
    Color(1.000, 0.220, 0.000),
    Color(1.000, 0.220, 0.000), // 1000 K
    Color(1.000, 0.278, 0.000),
    Color(1.000, 0.325, 0.000),
    Color(1.000, 0.365, 0.000),
    Color(1.000, 0.396, 0.000),
    Color(1.000, 0.427, 0.000),
    Color(1.000, 0.451, 0.000),
    Color(1.000, 0.475, 0.000),
    Color(1.000, 0.494, 0.000),
    Color(1.000, 0.514, 0.000),
    Color(1.000, 0.541, 0.071), // 2000 K
    Color(1.000, 0.557, 0.129),
    Color(1.000, 0.576, 0.173),
    Color(1.000, 0.596, 0.212),
    Color(1.000, 0.616, 0.247),
    Color(1.000, 0.631, 0.282),
    Color(1.000, 0.647, 0.310),
    Color(1.000, 0.663, 0.341),
    Color(1.000, 0.678, 0.369),
    Color(1.000, 0.694, 0.396),
    Color(1.000, 0.706, 0.420), // 3000 K
    Color(1.000, 0.722, 0.447),
    Color(1.000, 0.733, 0.471),
    Color(1.000, 0.745, 0.494),
    Color(1.000, 0.757, 0.518),
    Color(1.000, 0.769, 0.537),
    Color(1.000, 0.780, 0.561),
    Color(1.000, 0.788, 0.580),
    Color(1.000, 0.800, 0.600),
    Color(1.000, 0.808, 0.624),
    Color(1.000, 0.820, 0.639), // 4000 K
    Color(1.000, 0.827, 0.659),
    Color(1.000, 0.835, 0.678),
    Color(1.000, 0.843, 0.694),
    Color(1.000, 0.851, 0.714),
    Color(1.000, 0.859, 0.729),
    Color(1.000, 0.867, 0.745),
    Color(1.000, 0.875, 0.761),
    Color(1.000, 0.882, 0.776),
    Color(1.000, 0.890, 0.792),
    Color(1.000, 0.894, 0.808), // 5000 K
    Color(1.000, 0.902, 0.824),
    Color(1.000, 0.910, 0.835),
    Color(1.000, 0.914, 0.851),
    Color(1.000, 0.922, 0.863),
    Color(1.000, 0.925, 0.878),
    Color(1.000, 0.933, 0.890),
    Color(1.000, 0.937, 0.902),
    Color(1.000, 0.941, 0.914),
    Color(1.000, 0.949, 0.925),
    Color(1.000, 0.953, 0.937), // 6000 K
    Color(1.000, 0.957, 0.949),
    Color(1.000, 0.961, 0.961),
    Color(1.000, 0.965, 0.969),
    Color(1.000, 0.973, 0.984),
    Color(1.000, 0.976, 0.992),
    Color(0.996, 0.976, 1.000),
    Color(0.988, 0.969, 1.000),
    Color(0.976, 0.965, 1.000),
    Color(0.969, 0.961, 1.000),
    Color(0.961, 0.953, 1.000), // 7000 K
    Color(0.953, 0.949, 1.000),
    Color(0.941, 0.945, 1.000),
    Color(0.937, 0.941, 1.000),
    Color(0.929, 0.937, 1.000),
    Color(0.922, 0.933, 1.000),
    Color(0.914, 0.929, 1.000),
    Color(0.906, 0.925, 1.000),
    Color(0.902, 0.922, 1.000),
    Color(0.894, 0.918, 1.000),
    Color(0.890, 0.914, 1.000), // 8000 K
    Color(0.882, 0.910, 1.000),
    Color(0.878, 0.906, 1.000),
    Color(0.871, 0.902, 1.000),
    Color(0.867, 0.902, 1.000),
    Color(0.863, 0.898, 1.000),
    Color(0.855, 0.898, 1.000),
    Color(0.851, 0.890, 1.000),
    Color(0.847, 0.890, 1.000),
    Color(0.843, 0.886, 1.000),
    Color(0.839, 0.882, 1.000), // 9000 K
    Color(0.831, 0.882, 1.000),
    Color(0.827, 0.878, 1.000),
    Color(0.824, 0.875, 1.000),
    Color(0.820, 0.875, 1.000),
    Color(0.816, 0.871, 1.000),
    Color(0.812, 0.867, 1.000),
    Color(0.812, 0.867, 1.000),
    Color(0.808, 0.863, 1.000),
    Color(0.804, 0.863, 1.000),
    Color(0.812, 0.855, 1.000), // 10000 K 
    Color(0.812, 0.855, 1.000),
    Color(0.808, 0.851, 1.000),
    Color(0.804, 0.851, 1.000),
    Color(0.800, 0.847, 1.000),
    Color(0.800, 0.847, 1.000),
    Color(0.796, 0.843, 1.000),
    Color(0.792, 0.843, 1.000),
    Color(0.792, 0.839, 1.000),
    Color(0.788, 0.839, 1.000),
    Color(0.784, 0.835, 1.000), // 11000 K
    Color(0.784, 0.835, 1.000),
    Color(0.780, 0.831, 1.000),
    Color(0.776, 0.831, 1.000),
    Color(0.776, 0.831, 1.000),
    Color(0.773, 0.827, 1.000),
    Color(0.773, 0.827, 1.000),
    Color(0.773, 0.824, 1.000),
    Color(0.769, 0.824, 1.000),
    Color(0.765, 0.824, 1.000),
    Color(0.765, 0.820, 1.000), // 12000 K
  ];

  #[inline(always)]
  pub fn normalize(&self) -> Self {
    let max = self.0.max(self.1).max(self.2);
    if max == 0.0 {
      Color::BLACK
    } else {
      Color(self.0 / max, self.1 / max, self.2 / max)
    }
  }

  #[inline(always)]
  pub fn probabilize(&self) -> Self {
    fn clip(a: Flt) -> Flt {
      if a < 0.0 { 0.0 } else { a }
    }

    let r1 = clip(self.0);
    let g1 = clip(self.1);
    let b1 = clip(self.2);
    let mag = r1 + g1 + b1;
    if mag == 0.0 {
      Color(1.0/3.0, 1.0/3.0, 1.0/3.0)
    } else {
      Color(r1/mag, g1/mag, b1/mag)
    }
  }

  #[inline(always)]
  pub fn decide_wavelength(&self, p: Flt) -> Wavelength {
    if p < self.0 {
      Wavelength::Red
    } else if p < self.0 + self.1 {
      Wavelength::Green
    } else {
      Wavelength::Blue
    }
  }

  #[inline(always)]
  pub fn wavelength(&self, w: Wavelength) -> Flt {
    match w {
      Wavelength::Red   => self.0,
      Wavelength::Green => self.1,
      Wavelength::Blue  => self.2,
    }
  }

  pub fn exp(&self, e: Flt) -> Color {
    if *self == Self::WHITE {
      Self::WHITE
    } else {
      Color(self.0.powf(e), self.1.powf(e), self.2.powf(e))
    }
  }

  pub fn is_lower(&self, c: &Color) -> bool {
    self.0 < c.0 && self.1 < c.1 && self.2 < c.2
  }

  // cos: cos(theta), theta is angle between N and L
  pub fn approx_reflectance(&self, cos: Flt) -> Color {
    /*
    fn schlick_c(f0: Flt, cos: Flt) -> Flt {
      f0 + (1.0 - f0) * (1.0 - cos).powf(5.0)
    }
    */
    Color(approx_reflectance(self.0, cos), approx_reflectance(self.1, cos), approx_reflectance(self.2, cos))
  }

  #[inline(always)]
  pub fn blend(&self, c2: &Color, ratio: Flt) -> Color {
    let ratio2 = 1.0 - ratio;
    Color(self.0 * ratio2 + c2.0 * ratio, self.1 * ratio2 + c2.1 * ratio, self.2 * ratio2 + c2.2 * ratio)
  }

  #[inline(always)]
  pub fn relative_ior(&self, ior2: &Color) -> Color {
    Color(Self::_relative_ior(self.0, ior2.0), Self::_relative_ior(self.1, ior2.1), Self::_relative_ior(self.2, ior2.2))
  }

  #[inline(always)]
  pub fn relative_ior_wavelength(&self, ior2: &Color, wl: Wavelength) -> Flt {
    match wl {
      Wavelength::Red   => Self::_relative_ior(self.0, ior2.0),
      Wavelength::Green => Self::_relative_ior(self.1, ior2.1),
      Wavelength::Blue  => Self::_relative_ior(self.2, ior2.2),
    }
  }

  #[inline(always)]
  pub fn relative_ior_average(&self, ior2: &Color) -> Flt {
    let aior1 = (self.0 + self.1 + self.2) / 3.0;
    let aior2 = (ior2.0 + ior2.1 + ior2.2) / 3.0;
    Self::_relative_ior(aior1, aior2)
  }

  /*
    Physics Lows
      IoR: Index of Refraction, 屈折率
      relative of IoR: 相対屈折率
        n1: IoR of src
        n2: IoR of dst
        eta(η) = n2 / n1
  */
  // eta = n2 / n1
  #[inline(always)]
  fn _relative_ior(ior1: Flt, ior2: Flt) -> Flt {
    match ior1 {
      0.0 => 1.0,
      _   => ior2 / ior1,
    }
  }

}

// Trait supports for Color

impl fmt::Display for Color {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "COL[{},{},{}]", self.0, self.1, self.2)
  }
}

impl FromStr for Color {
  type Err = ParseFloatError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let re = Regex::new(r"^COL\[(\S+?),(\S+?),(\S+?)\]$").unwrap();
    let caps = re.captures(s).unwrap();
    let r = caps[1].parse::<Flt>()?;
    let g = caps[2].parse::<Flt>()?;
    let b = caps[3].parse::<Flt>()?;
    Ok(Color::new(r, g, b))
  }
}

impl Neg for Color {
  type Output = Self;

  #[inline(always)]
  fn neg(self) -> Self::Output {
    Color(1.0 - self.0, 1.0 - self.1, 1.0 - self.2)
  }
}

impl Add for Color {
  type Output = Self;

  #[inline(always)]
  fn add(self, target: Self) -> Self::Output {
    Color(self.0 + target.0, self.1 + target.1, self.2 + target.2)
  }
}

impl Sub for Color {
  type Output = Self;

  #[inline(always)]
  fn sub(self, target: Self) -> Self::Output {
    Color(self.0 - target.0, self.1 - target.1, self.2 - target.2)
  }
}

impl Mul<Color> for Color {
  type Output = Self;
  
  #[inline(always)]
  fn mul(self, col: Color) -> Self::Output {
    Color(self.0 * col.0, self.1 * col.1, self.2 * col.2)
  }
}

impl Mul<Flt> for Color {
  type Output = Self;

  #[inline(always)]
  fn mul(self, s: Flt) -> Self::Output {
    Color(self.0 * s, self.1 * s, self.2 * s)
  }
}

impl Mul<Color> for Flt {
  type Output = Color;

  #[inline(always)]
  fn mul(self, col: Color) -> Self::Output {
    Color(self * col.0, self * col.1, self * col.2)
  }
}

#[test]
fn test_color() {
  // new
  let c1 = Color(0.4, 0.78, 1.5);
  assert_eq!(format!("{}", c1), "COL[0.4,0.78,1.5]");
  let c11 = Color::new(0.4, 0.78, 1.5);
  assert_eq!(c1, c11);

  let c2 = c1.normalize();
  assert_eq!(c2, Color::new(0.4 / 1.5, 0.78 / 1.5, 1.5 / 1.5));
  assert_eq!(c2.0 + c2.1 + c2.2, 0.4 / 1.5 + 0.78 / 1.5 + 1.5 / 1.5);
  let c21 = Color::new_normalize(0.4, 0.78, 1.5);
  assert_eq!(c2, c21);

  let c2 = c1.probabilize();
  assert_eq!(c2.0 + c2.1 + c2.2, 1.0);
  assert_eq!(c2, Color::new(0.4 / (0.4+0.78+1.5), 0.78 / (0.4+0.78+1.5), 1.5 / (0.4+0.78+1.5)));
  let c21 = Color::new_probabilize(0.4, 0.78, 1.5);
  assert_eq!(c2, c21);

  // new_by_kelvin
  let c3 = Color::new_by_kelvin(4000.0);
  assert_eq!(c3, Color(0.4067650069018957, 0.328309180155668, 0.2649258129424364));
  let c3 = Color::new_by_kelvin2(4000.0);
  assert_eq!(c3, Color(0.40556279942541984, 0.334143895006124, 0.2602933055684562));
  let c3 = Color::new_by_kelvin3(4000.0);
  assert_eq!(c3, Color(0.40666937779585205, 0.33346888979259864, 0.2598617324115495));

  let c3 = Color::new_by_kelvin(1900.0);
  assert_eq!(c3, Color(0.6593131441917437, 0.3406868558082563, 0.0));
  let c3 = Color::new_by_kelvin2(1900.0);
  assert_eq!(c3, Color(0.6602706620002318, 0.3397293379997681, 0.0));
  let c3 = Color::new_by_kelvin3(1900.0);
  assert_eq!(c3, Color(0.6605019815059445, 0.3394980184940555, 0.0));

  let c3 = Color::new_by_kelvin(6500.0);
  assert_eq!(c3, Color(0.3359011141561401, 0.33472886389070683, 0.329370021953153));
  let c3 = Color::new_by_kelvin2(6500.0);
  assert_eq!(c3, Color(0.33663266161341554, 0.32787117091709067, 0.3354961674694938));
  let c3 = Color::new_by_kelvin3(6500.0);
  assert_eq!(c3, Color(0.33692722371967654, 0.3288409703504043, 0.33423180592991913));

  // decide_wavelength
  assert_eq!(c2.decide_wavelength(-0.5), Wavelength::Red);
  assert_eq!(c2.decide_wavelength(0.0), Wavelength::Red);
  assert_eq!(c2.decide_wavelength(0.148), Wavelength::Red);
  assert_eq!(c2.decide_wavelength(0.15), Wavelength::Green);
  assert_eq!(c2.decide_wavelength(0.44), Wavelength::Green);
  assert_eq!(c2.decide_wavelength(0.45), Wavelength::Blue);
  assert_eq!(c2.decide_wavelength(1.1), Wavelength::Blue);
  // wavelength
  assert_eq!(c1.wavelength(Wavelength::Red)  , 0.4);
  assert_eq!(c1.wavelength(Wavelength::Green), 0.78);
  assert_eq!(c1.wavelength(Wavelength::Blue) , 1.5);

  // exp
  let c4 = Color::WHITE;
  assert_eq!(c4.exp(3.5), Color(1.0, 1.0, 1.0));
  assert_eq!(c4.exp(0.0), Color(1.0, 1.0, 1.0));
  let c4 = Color::BLACK;
  assert_eq!(c4.exp(3.5), Color(0.0, 0.0, 0.0));
  assert_eq!(c4.exp(0.0), Color(1.0, 1.0, 1.0));
  let c4 = Color(0.3, 0.5, 0.2);
  assert_eq!(c4.exp(3.5), Color(0.014788509052639483, 0.08838834764831845, 0.003577708763999664));
  // is_lower
  assert!(Color(0.1, 0.2, 0.3).is_lower(&Color(0.11, 0.21, 0.31)) == true);
  assert!(Color(0.1, 0.2, 0.3).is_lower(&Color(0.09, 0.21, 0.31)) == false);
  assert!(Color(0.1, 0.2, 0.3).is_lower(&Color(0.11, 0.19, 0.31)) == false);
  assert!(Color(0.1, 0.2, 0.3).is_lower(&Color(0.11, 0.21, 0.29)) == false);
  assert!(Color(0.1, 0.2, 0.3).is_lower(&Color(0.09, 0.19, 0.31)) == false);
  assert!(Color(0.1, 0.2, 0.3).is_lower(&Color(0.11, 0.19, 0.29)) == false);
  assert!(Color(0.1, 0.2, 0.3).is_lower(&Color(0.09, 0.21, 0.29)) == false);
  // blend
  assert_eq!(Color::WHITE.blend(&Color::BLACK, 0.5), Color(0.5, 0.5, 0.5));
  let cb1 = Color(0.5, 0.3, 1.0);
  let cb2 = Color(0.8, 0.7, 0.3);
  assert_eq!(cb1.blend(&cb2, 0.3), Color(0.5 * 0.7 + 0.8 * 0.3, 0.3 * 0.7 + 0.7 * 0.3, 1.0 * 0.7 + 0.3 * 0.3));
  // approx_reflectance
  let cos = f64::cos(PI / 3.0);
  let c6 = Color::BLACK;
  assert_eq!(c6.approx_reflectance(1.0), Color::BLACK); // 0
  let r = 0.0 + (1.0 - 0.0) * (1.0 - cos).powf(5.0);
  assert_eq!(c6.approx_reflectance(cos), Color(r, r, r)); // 45
  assert_eq!(c6.approx_reflectance(0.0), Color::WHITE); // 90

  let c6 = Color::WHITE;
  assert_eq!(c6.approx_reflectance(1.0), Color::WHITE); // 0
  let r = 1.0 + (1.0 - 1.0) * (1.0 - cos).powf(5.0);
  assert_eq!(c6.approx_reflectance(cos), Color(r, r, r)); // 45
  assert_eq!(c6.approx_reflectance(0.0), Color::WHITE); // 90

  let c6 = Color(0.5, 0.4, 0.1);
  assert_eq!(c6.approx_reflectance(1.0), c6); // 0
  let rr = 0.5 + (1.0 - 0.5) * (1.0 - cos).powf(5.0);
  let rg = 0.4 + (1.0 - 0.4) * (1.0 - cos).powf(5.0);
  let rb = 0.1 + (1.0 - 0.1) * (1.0 - cos).powf(5.0);
  assert_eq!(c6.approx_reflectance(cos), Color(rr, rg, rb)); // 45
  assert_eq!(c6.approx_reflectance(0.0), Color::WHITE); // 90
  // relative_ior
  assert_eq!(Color::_relative_ior(0.0, 0.0), 1.0);
  assert_eq!(Color::_relative_ior(0.0, 1.0), 1.0);
  assert_eq!(Color::_relative_ior(0.0, 5.0), 1.0);

  assert_eq!(Color::_relative_ior(1.0, 1.0), 1.0);
  assert_eq!(Color::_relative_ior(1.0, 1.5), 1.5);
  assert_eq!(Color::_relative_ior(1.0, 5.0), 5.0);
  assert_eq!(Color::_relative_ior(1.5, 1.0), 1.0 / 1.5);
  assert_eq!(Color::_relative_ior(1.5, 1.5), 1.0);
  assert_eq!(Color::_relative_ior(1.5, 5.0), 5.0 / 1.5);

  // Display
  let c7 = Color(0.8, 0.2, 0.65);
  assert_eq!(format!("{}", c7), "COL[0.8,0.2,0.65]");
  // FromStr
  let c8 = Color::from_str("COL[0.4,0.5,0.1]");
  assert_eq!(c8.unwrap(), Color(0.4, 0.5, 0.1));
  let c8 = Color::from_str("COL[0.8,1.0,0.2]");
  assert_eq!(c8.unwrap(), Color(0.8, 1.0, 0.2));
  let c8 = Color::from_str("COL[0.8,1.0,0.2]");
  let c81 = Color::from_str(&format!("{}", c8.unwrap()));
  assert_eq!(c81.unwrap(), Color(0.8, 1.0, 0.2));


  // operators
  let c4 = Color(0.8, 0.5, 0.3);
  let c5 = Color(0.2, 0.3, 0.8);
  // neg
  let c41 = -c4;
  assert!((c41.0 - (1.0 - c4.0)).abs() < 0.00001);
  assert!((c41.1 - (1.0 - c4.1)).abs() < 0.00001);
  assert!((c41.2 - (1.0 - c4.2)).abs() < 0.00001);
  // add
  assert_eq!(c4 + c5, Color(0.8 + 0.2, 0.5 + 0.3, 0.3 + 0.8));
  // sub
  assert_eq!(c4 - c5, Color(0.8 - 0.2, 0.5 - 0.3, 0.3 - 0.8));
  // mul (Color x Flt)
  assert_eq!(c5 * 2.0, Color(0.2 * 2.0, 0.3 * 2.0, 0.8 * 2.0));
  // mul (Flt x Color)
  assert_eq!(2.0 * c5, Color(0.2 * 2.0, 0.3 * 2.0, 0.8 * 2.0));
  // mul (Color x Color)
  assert_eq!(c4 * c5, Color(0.8 * 0.2, 0.5 * 0.3, 0.3 * 0.8));

}

// Radiance ----------------------------

/*
  光束の単位「 lm 」について

  大きな考え方としては、「放射束」を人間の眼で見たときの「明るさ」が「光束」
  なのですが、この関係をもう少し厳密に説明します。正確には、放射量（ワット
  単位）から測光量（ルーメン単位）への変換には、最大視感効果度 Km と呼ばれる
  変換定数を掛ける必要があります。
  
     Km ＝ 683 [ lm / W ]

  cf: https://www.ccs-inc.co.jp/guide/column/light_color/vol04.html

--
  ８畳間では3300-4300ルーメンぐらい。flux=5.0だと上の換算値から、

    5.0[W] x 683[lm/W] = 3415[lm]
  
  となり、妥当な数字。

--
  作例の天井ライトは1.34[m]四方、5.0[W]である。光は下面のみから放射されるため
  半球に広がる(1/2π[1/sr])となる。よってこのライトの放射輝度は、

    5.0 / 2π / (1.34^2) = 0.4421 [W/sr/m^2]

    ※ これを光の三原色で均等割するのか、それぞれがこの値か？
*/

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Radiance(pub Flt, pub Flt, pub Flt);

impl Radiance {
  pub const ZERO: Radiance = Radiance(0.0, 0.0, 0.0);
  pub const ONE : Radiance = Radiance(1.0, 1.0, 1.0);

  #[inline(always)]
  pub fn from_color(color: &Color, intensity: Flt) -> Self {
    Radiance(color.0 * intensity, color.1 * intensity, color.2 * intensity)
  }

  #[inline(always)]
  pub fn rabs(self) -> Radiance {
    fn _rabs(d: Flt) -> Flt {
      if d < 0.0 {
        -d
      } else {
        d
      }
    }

    Radiance(_rabs(self.0), _rabs(self.1), _rabs(self.2))
  }

  #[inline(always)]
  pub fn r(self) -> Flt {
    self.0
  }

  #[inline(always)]
  pub fn g(self) -> Flt {
    self.1
  }

  #[inline(always)]
  pub fn b(self) -> Flt {
    self.2
  }

}

impl Add for Radiance {
  type Output = Radiance;

  #[inline(always)]
  fn add(self, target: Self) -> Self::Output {
    Radiance(self.0 + target.0, self.1 + target.1, self.2 + target.2)
  }
}

impl Sub for Radiance {
  type Output = Radiance;

  #[inline(always)]
  fn sub(self, target: Self) -> Self::Output {
    Radiance(self.0 - target.0, self.1 - target.1, self.2 - target.2)
  }
}

impl Mul<Radiance> for Flt {
  type Output = Radiance;

  #[inline(always)]
  fn mul(self, rad: Radiance) -> Self::Output {
    Radiance(self * rad.0, self * rad.1, self * rad.2)
  }
}

// Color x Radiance
impl Mul<Radiance> for Color {
  type Output = Radiance;

  #[inline(always)]
  fn mul(self, rad: Radiance) -> Self::Output {
    Radiance(self.0 * rad.0, self.1 * rad.1, self.2 * rad.2)
  }
}

impl BasicMatrix for Radiance {

  #[inline(always)]
  fn norm(&self) -> Flt {
    let r = self.rabs();
    r.0 + r.1 + r.2
  }

  #[inline(always)]
  fn near(&self, target: &Radiance) -> bool {
    (*self - *target).norm() < NEARLY0
  }
}

#[test]
fn test_radiance() {
  // consts
  assert_eq!(Radiance::ZERO, Radiance(0.0, 0.0, 0.0));
  assert_eq!(Radiance::ONE , Radiance(1.0, 1.0, 1.0));
  // from_color
  let col = Color(0.3, 0.5, 0.2);
  assert_eq!(Radiance::from_color(&col, 0.1), Radiance(0.3 * 0.1, 0.5 * 0.1, 0.2 * 0.1));
  // rabs
  let r1 = Radiance(-1.5, 2.3, 0.9);
  assert_eq!(r1.rabs(), Radiance(1.5, 2.3, 0.9));
  let r1 = Radiance(1.5, -2.3, 0.9);
  assert_eq!(r1.rabs(), Radiance(1.5, 2.3, 0.9));
  let r1 = Radiance(1.5, 2.3, -0.9);
  assert_eq!(r1.rabs(), Radiance(1.5, 2.3, 0.9));
  // elems
  assert_eq!(r1.r(), 1.5);
  assert_eq!(r1.g(), 2.3);
  assert_eq!(r1.b(), -0.9);

  // add
  let r2 = Radiance(5.2, -3.8, 9.5);
  assert_eq!(r1 + r2, Radiance(1.5 + 5.2, 2.3 + (-3.8), -0.9 + 9.5));
  // sub
  assert_eq!(r1 - r2, Radiance(1.5 - 5.2, 2.3 - (-3.8), -0.9 - 9.5));
  // mul
  assert_eq!(1.3 * r1, Radiance(1.3 * 1.5, 1.3 * 2.3, 1.3 * -0.9));
  let c1 = Color(0.2, 0.5, 0.3);
  assert_eq!(c1 * r1, Radiance(0.2 * 1.5, 0.5 * 2.3, 0.3 * -0.9));

  // norm
  assert_eq!(r1.norm(), 1.5 + 2.3 + 0.9);
  assert_eq!(r2.norm(), 5.2 + 3.8 + 9.5);


}


// PhotonFilter ------------------------

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PhotonFilter {
  Nonfilter,
  Conefilter,
  Gaussfilter,
}

// Photon ------------------------------

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Photon(pub Wavelength, pub Ray);

impl Photon {

  #[inline(always)]
  pub fn new(wl: Wavelength, ray: &Ray) -> Self {
    Photon(wl, *ray)
  }

  #[inline(always)]
  pub fn dummy(pos: &Position3) -> Photon {
    Photon(Wavelength::Red, Ray::new(pos, &Vector3::EX).unwrap())
  }

  #[inline(always)]
  pub fn wl(self) -> Wavelength {
    self.0
  }

  #[inline(always)]
  pub fn ray(self) -> Ray {
    self.1
  }

  #[inline(always)]
  pub fn dir(self) -> Direction3 {
    self.1.dir
  }

  #[inline(always)]
  pub fn pos(self) -> Position3 {
    self.1.pos
  }

  // convert_to_info: ★実装をどうするか？逆向きである必要があるか？
  /*
  pub fn convert_to_info(self) -> Photon {

  }
  */

  pub fn squared_distance(self, target: &Photon) -> Flt {
    let dir = self.1.pos - target.1.pos;
    dir.square()
  }

  // おそらく、このcosの正負の制御で上のconvert_to_infoの実装要否がでてくる？
  // PhotonのRay
  pub fn to_radiance(self, nvec: &Direction3, pw: Flt) -> Radiance {
    let cos = self.ray().dir.dot(nvec);
    if cos < 0.0 {
      // Photon = PhotonInfo とする場合、法線ベクトルと光子の入射角が90度以上なら物体表面から光子が当たったことになる。
      match self.0 {
        // どのような角度から入射しても光子のエネルギーは変わらない。入射角が大きいほど暗くなる現象は光子の密度によって表現される。
        Wavelength::Red   => Radiance(pw, 0.0, 0.0),
        Wavelength::Green => Radiance(0.0, pw, 0.0),
        Wavelength::Blue  => Radiance(0.0, 0.0, pw),
      }
    } else {
      // 光子が物体裏面から当たったことになるので輝度はゼロ（カウントしない）。
      Radiance::ZERO
    }
  }

}

impl KdPoint for Photon {
  type Scalar = Flt;
  type Dim = typenum::U3;
  fn at(&self, k: usize) -> Flt {
    self.1.pos.at(k)
  }
}

#[test]
fn test_photon() {
  // new
  let p0 = Photon::new(Wavelength::Red, &Ray::new(&Vector3::O, &Vector3::EY).unwrap());
  assert_eq!(p0.wl(), Wavelength::Red);
  assert_eq!(p0.ray().pos, Vector3::O);
  assert_eq!(p0.ray().dir, Vector3::EY);
  let p1 = Photon::dummy(&Vector3::new(1.5, 0.4, -2.5));
  assert_eq!(p1.wl(), Wavelength::Red);
  assert_eq!(p1.ray().pos, Vector3::new(1.5, 0.4, -2.5));
  assert_eq!(p1.ray().dir, Vector3::EX);
  assert_eq!(p1.ray().pos, p1.pos());
  assert_eq!(p1.ray().dir, p1.dir());
  // squared_distance
  let p2 = Photon::dummy(&Vector3::new(1.6, 0.5, -2.6));
  assert!(p1.squared_distance(&p2) - (0.1 * 0.1 + 0.1 * 0.1 + 0.1 * 0.1) < NEARLY0);
  // to_radiance
  let nvec = Vector3::EY;
  let ppos = Vector3::O;
  let mut pdir = -Vector3::EY; // 真上から下への光子
  let mut p3 = Photon::new(Wavelength::Red, &Ray::new(&ppos, &pdir).unwrap());
  assert_eq!(p3.to_radiance(&nvec, 0.1), Radiance(0.1, 0.0, 0.0));
  p3 = Photon::new(Wavelength::Green, &Ray::new(&ppos, &pdir).unwrap());
  assert_eq!(p3.to_radiance(&nvec, 0.1), Radiance(0.0, 0.1, 0.0));
  p3 = Photon::new(Wavelength::Blue, &Ray::new(&ppos, &pdir).unwrap());
  assert_eq!(p3.to_radiance(&nvec, 0.1), Radiance(0.0, 0.0, 0.1));
  pdir = Vector3::EY;
  p3 = Photon::new(Wavelength::Red, &Ray::new(&ppos, &pdir).unwrap());
  assert_eq!(p3.to_radiance(&nvec, 0.1), Radiance::ZERO);
  pdir = Vector3::new(1.0, -1.0, 1.0).normalize().unwrap();
  p3 = Photon::new(Wavelength::Red, &Ray::new(&ppos, &pdir).unwrap());
  assert_eq!(p3.to_radiance(&nvec, 0.1), Radiance(0.1, 0.0, 0.0));

  /* Test for kd-tree (KdTree)
   */
  let dir = Vector3::new(1.0, -1.0, 0.0);
  let p4 = Photon::new(Wavelength::Red, &Ray::new(&Vector3::new(0.0, 1.0, 0.0), &dir).unwrap());
  let p5 = Photon::new(Wavelength::Green, &Ray::new(&Vector3::new(0.0, 1.0, 0.3), &dir).unwrap());
  let p6 = Photon::new(Wavelength::Blue, &Ray::new(&Vector3::new(0.0, 1.5, 0.5), &dir).unwrap());
  let p7 = Photon::new(Wavelength::Red, &Ray::new(&Vector3::new(0.0, 1.0, 1.0), &dir).unwrap());
  let tree: KdTree<Photon> = KdTree::build_by_ordered_float(vec![p4, p5, p6, p7]);
  let v1 = Vector3::new(0.0, 1.0, 0.0);
  let p8 = Photon::new(Wavelength::Blue, &Ray::new(&v1, &Vector3::EY).unwrap());
  assert_eq!(tree.within_radius(&p8, 0.0), Vec::<&Photon>::new());
  assert_eq!(tree.within_radius(&p8, 0.2), vec![&p4]);
  assert_eq!(tree.within_radius(&p8, 0.5), vec![&p4, &p5]);
  assert_eq!(tree.within_radius(&p8, 0.8), vec![&p6, &p4, &p5]);
  assert_eq!(tree.within_radius(&p8, 1.1), vec![&p6, &p4, &p5, &p7]);

}


// SUPPORT FUNCS FOR OPTICS ------------------------

// diffuse reflection
//

pub fn diffuse_reflection(nvec: &Direction3) -> Direction3 {
  let dir = generate_random_dir4();
  let cos = dir.dot(nvec);
  // 生成したベクトルVがNとのなす角90度未満ならそのまま、以上ならVを反転して返す
  if cos > 0.0 { dir } else { -dir }
}


// specular reflection
// IN : nvec  = Normal vector (from surface)
//      vvec  = eye direction (to surface)
// OUT: rvec  = reflection vector (from surface)
//      cos1  = (rvec, nvec)
// 条件: cos = (N, V) は負にならないといけない。万が一そうでなければNを返す。

pub fn specular_reflection(nvec: &Direction3, vvec: &Direction3) -> (Direction3, Flt) {
  let cos = -vvec.dot(nvec);  // -(E,N)
  if cos < 0.0 {
    return (*nvec, -cos)
  }

  let rvec = (*vvec + 2.0 * cos * *nvec).normalize();
  match rvec {
    Some(vec) => (vec, cos),
    None  => (*nvec, 0.0),
  }
}

// specular_rafraction
// cf. http://kanamori.cs.tsukuba.ac.jp/jikken/inner/reflection_refraction.pdf
//
// IN : 
//   N (nvec) = Normal vector (from surface)
//   L (vvec) = eye direction (to surface)
//   η (eta)  = relative of IoR (= n2 / n1)
//   c (cos)  = -(L.N)
//
//   T = 1/η {L + (c - g) N}
//
//     c (cos) = -(L.N)
//     g = √(η^2 + c^2 - 1)
//     η = η2 / η1 
//
// OUT: tvec  = T, refraction vector (from surface)
//      cos2  = (T, -N) = g / η

pub fn specular_refraction(nvec: &Direction3, vvec: &Direction3, eta: Flt, cos: Flt) -> (Option<Direction3>, Flt) {
  if cos < 0.0 || eta <= 0.0 { // 入力値範囲エラー
    return (None, -1.0)
  }

  let g0 = eta * eta + cos * cos - 1.0;
  if g0 < 0.0 { // 全反射
    (None, 0.0)
  } else {
    let g = f64::sqrt(g0);
    let tvec = ((1.0 / eta) * (*vvec + (cos - g) * *nvec)).normalize();
    match tvec {
      Some(vec) => (Some(vec), g / eta),
      None => (None, -1.0),
    }
  }

}

// Snell's low
// IN : r_ior = relative of IoR (n = n2/n1)
//      nvec  = Normal vector (from surface)
//      vvec  = eye direction (to surface)
// OUT: R  reflection dir
//      T  refraction dir
//      cos1  reflection cosine
//      cos2  refraction cosine

pub fn snell(r_ior: Flt, nvec: &Direction3, vvec: &Direction3) -> (Direction3, Option<Direction3>, Flt, Flt) {
  let c1 = vvec.dot(nvec);
  let r = (*vvec - (2.0 * c1) * *nvec).normalize().unwrap();
  if c1 > 0.0 || r_ior <= 0.0 { // 入力値範囲エラー
    return (r, None, c1, 0.0)
  }
  let n = r_ior * r_ior;
  let g = n + c1 * c1 - 1.0;
  if g <= 0.0 {
    // 全反射
    (r, None, c1, 0.0)
  } else {
    let a = -c1 - f64::sqrt(g);
    let t = (r_ior * (*vvec + a * *nvec)).normalize().unwrap();
    let c2 = f64::sqrt(1.0 - 1.0 / n * (1.0 - c1 * c1));
    (r, Some(t), c1, c2)
  }
}

// Schlick's approximation: シュリック近似式を用いる
pub fn approx_reflectance(f0: Flt, cos: Flt) -> Flt {
  f0 + (1.0 - f0) * (1.0 - cos).powf(5.0)
}

/*
// glossyな表面の反射ベクトルの求め方
//   http://www.raytracegroundup.com/downloads/Chapter25.pdf
//   https://cg.informatik.uni-freiburg.de/course_notes/graphics2_08_renderingEquation.pdf
//   https://graphics.cg.uni-saarland.de/courses/ris-2018/slides/09_BRDF_LightSampling.pdf
//   x = cos(2 pi xi2) sqrt(1 - xi1^(2/n+1))
//   y = xi1^(1/n+1)
//   z = sin(2 pi xi2) sqrt(1 - xi1^(2/n+1))
//     where xi1 = cos (w.r) ^ 10 ^ (5 x (1 - sqrt(roughness)))

pub fn reflection_glossy(nvec: &Direction3, rvec: &Direction3, pw: Flt) -> Direction3 {
  //let uvec0 = Vector3::new(0.00424, 1.0, 0.00764).cross(&rvec).normalize();
  let uvec0 = (*rvec * Vector3::new(0.00424, 1.0, 0.00764)).normalize();
  let uvec = match uvec0 {
    Some(v) => v,
    None    => {
      Vector3::new(1.0, 0.00424, 0.00764).cross(&rvec).normalize().unwrap()
    },
  };
  let vvec = uvec.cross(rvec);
  let mut rng = rand::thread_rng();
  // 試行として入射角に応じて反射ベクトルの分散度を変化させる(cosを掛ける)
  let c0 = nvec.dot(&rvec);
  //if c0 < 0.0 { eprintln!("COS is Minus: {}", c0)}
  let xi0 = rng.gen_range(0.0, 1.0) as Flt;
  let xi1 = xi0.powf(pw * c0);
  let xi2 = 2.0 * f64::consts::PI * rng.gen_range(0.0, 1.0);

  let x = f64::cos(xi2) * f64::sqrt(1.0 - xi1 * xi1);
  let y = xi1;
  let z = f64::sin(xi2) * f64::sqrt(1.0 - xi1 * xi1);

  let mut wi = x * uvec + y * *rvec + z * vvec;
  if nvec.dot(&wi) < 0.0 {
    wi = -x * uvec + y * *rvec - z * vvec;
  }
  match wi.normalize() {
    Some(v) => v,
    None    => Vector3::EX,
  }
}
*/

#[test]
fn test_support_func() {
  let nvec = Vector3::EY;
  let mut vvec: Vector3;

  // diffuse_reflection
  for _i in 0..500 {
    vvec = diffuse_reflection(&nvec);
    assert!(nvec.dot(&vvec) > 0.0);
  }
  // specular_reflection
  vvec = -Vector3::EY;
  let mut rvec = specular_reflection(&nvec, &vvec);
  assert_eq!(rvec, (Vector3::EY, 1.0));
  vvec = Vector3::EZ;
  rvec = specular_reflection(&nvec, &vvec);
  assert_eq!(rvec, (Vector3::EZ, 0.0));
  vvec = Vector3::new(1.0, -1.0, 1.0).normalize().unwrap();
  rvec = specular_reflection(&nvec, &vvec);
  assert_eq!(rvec, (Vector3::new(1.0, 1.0, 1.0).normalize().unwrap(), 1.0 / f64::sqrt(3.0)));
  // specular_refraction
  let (rvec, c2) = specular_refraction(&nvec, &vvec, -0.5, 1.0);
  assert_eq!((rvec, c2), (None, -1.0));
  let (rvec, c2) = specular_refraction(&nvec, &vvec, 0.0, 1.0);
  assert_eq!((rvec, c2), (None, -1.0));
  let (rvec, c2) = specular_refraction(&nvec, &vvec, -0.5, -0.1);
  assert_eq!((rvec, c2), (None, -1.0));
  let (rvec, c2) = specular_refraction(&nvec, &vvec, 0.5, -0.1);
  assert_eq!((rvec, c2), (None, -1.0));
  let (rvec, c2) = specular_refraction(&nvec, &vvec, 0.1, 0.1);
  assert_eq!((rvec, c2), (None, 0.0));
  vvec = -Vector3::EY;
  let mut c = -nvec.dot(&vvec);
  let (rvec, c2) = specular_refraction(&nvec, &vvec, 1.0, c);
  assert_eq!((rvec, c2), (Some(-Vector3::EY), 1.0));
  let (rvec, c2) = specular_refraction(&nvec, &vvec, 1.5, c);
  assert_eq!((rvec, c2), (Some(-Vector3::EY), 1.0));
  let (rvec, c2) = specular_refraction(&nvec, &vvec, 2.0, c);
  assert_eq!((rvec, c2), (Some(-Vector3::EY), 1.0));
  vvec = Vector3::new(1.0, -1.0, 0.0).normalize().unwrap();
  c = -nvec.dot(&vvec);
  let (rvec, c2) = specular_refraction(&nvec, &vvec, 1.0, c);
  assert!(vvec.near(&rvec.unwrap()));
  assert!((c - c2).abs() < NEARLY0);
  let (rvec, c2) = specular_refraction(&nvec, &vvec, 1.5, c);
  assert_eq!((rvec, c2), (Some(Vector3::new(0.4714045207910316, -0.8819171036881969, 0.0)), 0.8819171036881969));

  // snell
  vvec = -Vector3::EY;
  let res = snell(1.0, &nvec, &vvec);
  assert_eq!(res, (Vector3::EY, Some(-Vector3::EY), -1.0, 1.0));
  vvec = Vector3::EZ;
  let res = snell(1.0, &nvec, &vvec);
  assert_eq!(res, (Vector3::EZ, None, 0.0, 0.0));
  vvec = Vector3::new(1.0, -1.0, 1.0).normalize().unwrap();
  let res = snell(1.0, &nvec, &vvec);
  let c = nvec.dot(&vvec);
  assert_eq!(res, (Vector3::new(1.0, 1.0, 1.0).normalize().unwrap(), Some(vvec), c, -c));

  let res = snell(-0.5, &nvec, &vvec);
  assert_eq!(res, (Vector3::new(1.0, 1.0, 1.0).normalize().unwrap(), None, c, 0.0));
  let res = snell(0.0, &nvec, &vvec);
  assert_eq!(res, (Vector3::new(1.0, 1.0, 1.0).normalize().unwrap(), None, c, 0.0));
  let res = snell(0.1, &nvec, &vvec);
  assert_eq!(res, (Vector3::new(1.0, 1.0, 1.0).normalize().unwrap(), None, c, 0.0));
  vvec = Vector3::new(1.0, -1.0, 0.0).normalize().unwrap();
  let (_, rvec, _, c2) = snell(1.5, &nvec, &vvec);
  //let c = -f64::cos(f64::consts::PI / 4.0);
  assert_eq!(rvec, Some(Vector3::new(0.4714045207910316, -0.8819171036881969, 0.0)));
  assert_eq!(c2, 0.8819171036881968);
  /*
  // reflection_glossy
  let mut rvec = Vector3::EY;
  let mut wvec = reflection_glossy(&nvec, &rvec, 0.0);
  assert_eq!(wvec, Vector3::EY);
  rvec = Vector3::EY;
  wvec = reflection_glossy(&nvec, &rvec, 0.0);
  */

  // approx_reflectance
  assert_eq!(approx_reflectance(0.5, 1.0), 0.5);
  assert_eq!(approx_reflectance(0.5, 0.0), 1.0);

}


