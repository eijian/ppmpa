// algebra module

use core::num::ParseFloatError;
use std::f64;
use std::fmt;
use std::ops::Neg;
use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;
use std::ops::Div;
use std::str::*;

use rand::Rng;
use regex::Regex;

use super::*;

// 参考URLs
// https://scrapbox.io/nwtgck/Rustの演算子のオーバーロードで借用してmoveさせないようにする方法_-_std::ops::AddとかMulとか

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector3 {
  pub v: [Flt; 3],
}

impl fmt::Display for Vector3 {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "V3[{},{},{}]", self.v[0], self.v[1], self.v[2])
  }
}

// 文字列->構造体変換：参考URLs
// https://doc.rust-lang.org/std/str/trait.FromStr.html
// https://qiita.com/scivola/items/60141f262caa53983c3a
// https://text.baldanders.info/rust-lang/character-string-2/

impl FromStr for Vector3 {
  type Err = ParseFloatError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let re = Regex::new(r"^V3\[(\S+?),(\S+?),(\S+?)\]$").unwrap();
    let caps = re.captures(s).unwrap();
    let x = caps[1].parse::<Flt>()?;
    let y = caps[2].parse::<Flt>()?;
    let z = caps[3].parse::<Flt>()?;
    Ok(Vector3::new(x, y, z))
  }
}

impl Neg for Vector3 {
  type Output = Vector3;

  fn neg(self) -> Self::Output {
    Vector3 {v: [-self.v[0], -self.v[1], -self.v[2]] }
  }
}

impl Add for Vector3 {
  type Output = Vector3;

  fn add(self, target: Self) -> Self::Output {
    Vector3 {
      v: [self.v[0] + target.v[0], self.v[1] + target.v[1], self.v[2] + target.v[2]],
    }
  }
}

impl Sub for Vector3 {
  type Output = Vector3;

  fn sub(self, target: Self) -> Self::Output {
    Vector3 {
      v: [self.v[0] - target.v[0], self.v[1] - target.v[1], self.v[2] - target.v[2]],
    }
  }
}

impl Mul<Flt> for Vector3 {
  type Output = Vector3;

  fn mul(self, s: Flt) -> Self::Output {
    Vector3 {
      v: [self.v[0] * s, self.v[1] * s, self.v[2] * s],
    }
  }
}

impl Mul<Vector3> for Flt {
  type Output = Vector3;

  fn mul(self, vec: Vector3) -> Self::Output {
    Vector3 {
      v: [self * vec.v[0], self * vec.v[1], self * vec.v[2]],
    }
  }
}

impl Div<Flt> for Vector3 {
  type Output = Option<Vector3>;

  fn div(self, s: Flt) -> Self::Output {
    if s == 0.0 {
      None
    } else {
      Some(self * (1.0 / s))
    }
  }
}

pub trait BasicMatrix: Copy {
  fn norm(&self) -> Flt;
  fn near(&self, target: &Self) -> bool;
}

pub trait Vector: BasicMatrix + Copy + std::marker::Sized {
  fn dot(&self, target: &Self) -> Flt;
  fn square(&self) -> Flt {
    self.dot(self)
  }
  fn normalize(&self) -> Option<Self>;
}

impl BasicMatrix for Vector3 {
  fn norm(&self) -> Flt {
    f64::sqrt(self.square())
  }

  fn near(&self, target: &Self) -> bool {
    (*self - *target).norm() < NEARLY0
  }
}

impl Vector for Vector3 {
  fn dot(&self, target: &Self) -> Flt {
    self.v[0] * target.v[0] + self.v[1] * target.v[1] + self.v[2] * target.v[2]
  }

  fn normalize(&self) -> Option<Self> {
    let norm = self.norm();
    if norm == 0.0 {
      None
    } else {
      Some(*self * (1.0 / norm))
    }
  }

}

impl Vector3 {
  pub const O:  Vector3 = Vector3 {v: [0.0, 0.0, 0.0]};
  pub const EX: Vector3 = Vector3 {v: [1.0, 0.0, 0.0]};
  pub const EY: Vector3 = Vector3 {v: [0.0, 1.0, 0.0]};
  pub const EZ: Vector3 = Vector3 {v: [0.0, 0.0, 1.0]};

  pub fn new(x: Flt, y: Flt, z: Flt) -> Self {
    Vector3 {v: [x, y, z]}
  }

  pub fn cross(&self, target: &Self) -> Self {
    Vector3 {
      v: [self.v[1] * target.v[2] - target.v[1] * self.v[2],
          self.v[2] * target.v[0] - target.v[2] * self.v[0],
          self.v[0] * target.v[1] - target.v[0] * self.v[1]]
    }
  }
}

pub type Position3  = Vector3;

impl Position3 {
  pub fn new_pos(x: Flt, y: Flt, z: Flt) -> Self {
    Vector3 {v: [x, y, z]}
  }
}

pub type Direction3 = Vector3;

impl Direction3 {
  pub fn new_dir(x: Flt, y: Flt, z: Flt) -> Option<Self> {
    let v = Vector3 {v: [x, y, z]};
    v.normalize()
  }

  pub fn new_dir_from_angle(theta: Flt, phi: Flt) -> Option<Self> {
    let sint = f64::sin(theta);
    let x = sint * f64::cos(phi);
    let y = f64::cos(theta);
    let z = sint * f64::sin(phi);
    Self::new_dir(x, y, z)
  }
}

pub fn generate_random_dir() -> Direction3 {
  let mut rng = rand::thread_rng();
  loop {
    let x: Flt = rng.gen_range(-1.0, 1.0);
    let y: Flt = rng.gen_range(-1.0, 1.0);
    let z: Flt = rng.gen_range(-1.0, 1.0);
    let v = Vector3 {v: [x, y, z]};
    let len = v.norm();
    if 0.0 < len && len <= 1.0  {
      return v.normalize().unwrap();
    }
  }
}


// TESTS

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_vector3() {
    let v0 = Vector3::new(1.0, 2.0, 3.0);
    assert_eq!(format!("{}", v0), "V3[1,2,3]");
    let v1 = Vector3::new(1.0, 2.0, 3.0);
    assert_eq!(-v1, Vector3::new(-1.0, -2.0, -3.0));
    let v2 = Vector3::new(-1.0, 2.0, -3.0);
    assert_eq!(-v2, Vector3::new(1.0, -2.0, 3.0));
    let v3 = Vector3::new(1.0, -2.0, 3.0);
    assert_eq!(-v3, Vector3::new(-1.0, 2.0, -3.0));
    assert_eq!(v3.normalize(), Some(Vector3::new(0.2672612419124244, -0.5345224838248488, 0.8017837257372732)));

    assert_eq!(v1 + v2, Vector3::new(0.0, 4.0, 0.0));
    assert_eq!(v1 - v2, Vector3::new(2.0, 0.0, 6.0));
    assert_eq!(v1 + v2 - v2, Vector3::new(1.0, 2.0, 3.0));

    assert_eq!((v1 * 1.1).near(&Vector3::new(1.1, 2.2, 3.3)), true);
    assert_eq!(format!("{}", v1 * 1.1), "V3[1.1,2.2,3.3000000000000003]");
    assert_eq!((2.2 * v1).near(&Vector3::new(2.2, 4.4, 6.6)), true);

    let v4 = Vector3::from_str("V3[0.1,0.3,0.5]");
    let v42 = v4.unwrap();
    assert_eq!(v42, Vector3 {v: [0.1, 0.3, 0.5]});
  }
}


