/// physics

use std::fmt;
use std::ops::Neg;
use std::ops::Add;
use std::ops::Mul;
use rand::Rng;

use super::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Wavelength {
  Red,
  Green,
  Blue
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Color(pub Flt, pub Flt, pub Flt);

impl Color {
  pub const BLACK: Color = Color(0.0, 0.0, 0.0);
  pub const WHITE: Color = Color(1.0, 1.0, 1.0);

  pub fn new(r: Flt, g: Flt, b: Flt) -> Color {
    let c = Color(r, g, b);
    c.normalize()
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

  pub fn select_wavelength(&self, w: Wavelength) -> Flt {
    match w {
      Wavelength::Red   => self.0,
      Wavelength::Green => self.1,
      Wavelength::Blue  => self.2,
    }
  }

}

impl fmt::Display for Color {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "[{},{},{}]", self.0, self.1, self.2)
  }
}

impl Neg for Color {
  type Output = Self;

  fn neg(self) -> Self::Output {
    Color(1.0 -self.0, 1.0 -self.1, 1.0 -self.2)
  }

}

impl Add for Color {
  type Output = Self;

  fn add(self, target: Self) -> Self::Output {
    Color(self.0 + target.0, self.1 + target.1, self.2 + target.2)
  }
}

impl Mul<Flt> for Color {
  type Output = Self;

  fn mul(self, s: Flt) -> Self::Output {
    Color(self.0 * s, self.1 * s, self.2 * s)
  }
}

fn clip_color(a: Flt) -> Flt {
  if a < 0.0 {
    0.0
  } else{
    a
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

  }

  #[test]
  fn test_color() {
    let c1 = Color(0.4, 0.78, 1.0);
    assert_eq!(format!("{}", c1), "[0.4,0.78,1]");
    let c2 = -c1;
    assert!((c2.0 - 0.6).abs()  < 0.00001);
    assert!((c2.1 - 0.22).abs() < 0.00001);
    assert!((c2.2 - 0.0).abs()  < 0.00001);
    let c3 = c1.normalize();
    assert_eq!(c3.0 + c3.1 + c3.2, 1.0);
    assert_eq!(Color::new(0.4, 0.78, 1.0), c3);
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

