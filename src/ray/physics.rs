/// physics

use core::num::ParseFloatError;
use std::fmt;
use std::ops::Neg;
use std::ops::Add;
use std::ops::Mul;
use std::str::*;

use rand::Rng;
use regex::Regex;

use super::*;
use super::algebra::*;

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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Color(pub Flt, pub Flt, pub Flt);

impl Color {
  pub const BLACK: Color = Color(0.0, 0.0, 0.0);
  pub const WHITE: Color = Color(1.0, 1.0, 1.0);

  pub fn new(r: Flt, g: Flt, b: Flt) -> Color {
    // normalize() は明示的に実行する
    Color(r, g, b)
  }

  pub fn normalize(&self) -> Self {
    let r1 = clip_color(self.0);
    let g1 = clip_color(self.1);
    let b1 = clip_color(self.2);
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
    } else {
      if p < self.0 + self.1 {
        Wavelength::Green
      } else {
        Wavelength::Blue
      }
    }
  }

  #[inline(always)]
  pub fn wavelength(&self, w: &Wavelength) -> Flt {
    match w {
      Wavelength::Red   => self.0,
      Wavelength::Green => self.1,
      Wavelength::Blue  => self.2,
    }
  }

}

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

  fn neg(self) -> Self::Output {
    Color(1.0 - self.0, 1.0 - self.1, 1.0 - self.2)
  }

}

impl Add for Color {
  type Output = Self;

  fn add(self, target: Self) -> Self::Output {
    Color(self.0 + target.0, self.1 + target.1, self.2 + target.2)
  }
}

impl Mul<Color> for Color {
  type Output = Self;
  
  fn mul(self, col: Color) -> Self::Output {
    Color(self.0 * col.0, self.1 * col.1, self.2 * col.2)
  }
}

impl Mul<Flt> for Color {
  type Output = Self;

  fn mul(self, s: Flt) -> Self::Output {
    Color(self.0 * s, self.1 * s, self.2 * s)
  }
}

impl Mul<Color> for Flt {
  type Output = Color;

  fn mul(self, col: Color) -> Self::Output {
    Color(self * col.0, self * col.1, self * col.2)
  }
}

fn clip_color(a: Flt) -> Flt {
  if a < 0.0 {
    0.0
  } else{
    a
  }
}


// Physics Lows

// OUT: R  reflection dir
//      T  refraction dir
//      cos1  reflection cosine
//      cos2  refraction cosine

pub fn snell(r_ior: &Flt, nvec: &Direction3, vvec: &Direction3) -> (Direction3, Direction3, Flt, Flt) {
  let c1 = vvec.dot(nvec);
  let r = (*vvec - (2.0 * c1) * *nvec).normalize().unwrap();
  let n = r_ior * r_ior;
  let g = 1.0 / n + c1 * c1 - 1.0;
  if g > 0.0 {
    let a = -c1 - f64::sqrt(g);
    let t = (*r_ior * (*vvec + a * *nvec)).normalize().unwrap();
    let c2 = f64::sqrt(1.0 - n * (1.0 - c1 * c1));
    (r, t, c1, c2)
  } else {
    // 全反射
    (r, Vector3::O, c1, 0.0)
  }
}

pub fn schlick_color(f0: &Color, cos: &Flt) -> Color {
  Color::new(schlick(&f0.0, cos), schlick(&f0.1, cos), schlick(&f0.2, cos))
}

pub fn schlick(f0: &Flt, cos: &Flt) -> Flt {
  f0 + (1.0 - f0) * (1.0 - cos).powf(5.0)
}

pub fn relative_ior_color(ior1: &Color, ior2: &Color) -> Color {
  Color::new(
    relative_ior(&ior1.0, &ior2.0),
    relative_ior(&ior1.1, &ior2.1),
    relative_ior(&ior1.2, &ior2.2)
  )
}

pub fn relative_ior_wavelength(ior1: &Color, ior2: &Color, wl: &Wavelength) -> Flt {
  match wl {
    Wavelength::Red   => relative_ior(&ior1.0, &ior2.0),
    Wavelength::Green => relative_ior(&ior1.1, &ior2.1),
    Wavelength::Blue  => relative_ior(&ior1.2, &ior2.2),
  }
}

pub fn relative_ior_average(ior1: &Color, ior2: &Color) -> Flt {
  let aior1 = (ior1.0 + ior1.1 + ior1.2) / 3.0;
  let aior2 = (ior2.0 + ior2.1 + ior2.2) / 3.0;
  relative_ior(&aior1, &aior2)
}

fn relative_ior(ior1: &Flt, ior2: &Flt) -> Flt {
  match ior2 {
    0.0 => 1.0,
    _   => ior1 / ior2,
  }
}

//
// Russian Roulette

pub fn russian_roulette(ps: &[Flt]) -> usize {
  let mut rng = rand::thread_rng();
  let p: Flt = rng.gen_range(0.0, 1.0);

  check_under(ps, p)
}

fn check_under(ps: &[Flt], p: Flt) -> usize {
  let mut i: usize = 0;
  while i < ps.len() && p > ps[i] {
    i += 1;
  }
  i
}

// TESTS

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_wl() {
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
    let sx = "WL:aaa";
    match Wavelength::from_str(sx) {
      Ok(_) => assert!(false, format!("Str '{}' is invalid, but test is through!", sx)),
      Err(e) => assert_eq!(e, "invalid wavelength: WL:aaa"),
    }

  }

  #[test]
  fn test_color() {
    let c1 = Color(0.4, 0.78, 1.0);
    assert_eq!(format!("{}", c1), "COL[0.4,0.78,1]");
    let c2 = -c1;
    assert!((c2.0 - 0.6).abs()  < 0.00001);
    assert!((c2.1 - 0.22).abs() < 0.00001);
    assert!((c2.2 - 0.0).abs()  < 0.00001);
    let c3 = c1.normalize();
    assert_eq!(c3.0 + c3.1 + c3.2, 1.0);
    assert_eq!(c3, Color::new(0.1834862385321101, 0.35779816513761464, 0.4587155963302752));
    assert_eq!(c3.decide_wavelength(0.1), Wavelength::Red);
    assert_eq!(c3.decide_wavelength(0.3), Wavelength::Green);
    assert_eq!(c3.decide_wavelength(0.7), Wavelength::Blue);
    assert_eq!(c2.select_wavelength(Wavelength::Red)  , 0.6);
    assert_eq!(c2.select_wavelength(Wavelength::Green), 0.21999999999999997);
    assert_eq!(c2.select_wavelength(Wavelength::Blue) , 0.0);

    let c4 = Color(0.8, 0.5, 0.3);
    let c5 = Color(0.2, 0.3, 0.8);
    assert_eq!(c4 + c5, Color(1.0, 0.8, 1.1));
    assert_eq!(c5 * 2.0, Color(0.4, 0.6, 1.6));

    let c6 = Color::from_str("COL[0.4,0.5,0.1]");
    let c62 = c6.unwrap();
    assert_eq!(c62, Color(0.4,0.5,0.1));
    let c7 = Color::from_str("COL[0.8,1.0,0.2]");
    let c72 = c7.unwrap();
    assert_eq!(c72, Color(0.8,1.0,0.2));

    let c8 = Color::from_str(&format!("{}", c5));
    assert_eq!(c8.unwrap(), Color(0.2,0.3,0.8));


  }

  #[test]
  fn test_rr_check_under() {
    let ps = [0.1, 0.2, 0.3, 0.5, 0.8];
    assert_eq!(check_under(&ps, 0.03), 0);
    assert_eq!(check_under(&ps, 0.12), 1);
    assert_eq!(check_under(&ps, 0.28), 2);
    assert_eq!(check_under(&ps, 0.4 ), 3);
    assert_eq!(check_under(&ps, 0.64), 4);
    assert_eq!(check_under(&ps, 0.99), 5);    
  }

}

