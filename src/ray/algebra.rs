// algebra module

use std::f64;
use std::ops::Neg;
use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;
use std::ops::Div;


pub const NEARLY0: f64 = 0.0001;   // 100 micro meter

type Flt = f64;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector3(pub Flt, pub Flt, pub Flt);
type   Position3  = Vector3;
type   Direction3 = Vector3;

impl Neg for Vector3 {
  type Output = Self;

  fn neg(self) -> Self::Output {
    Vector3(-self.0, -self.1, -self.2)
  }
}

impl Add for Vector3 {
  type Output = Self;

  fn add(self, target: Self) -> Self::Output {
    Vector3(self.0 + target.0, self.1 + target.1, self.2 + target.2)
  }
}

impl Sub for Vector3 {
  type Output = Self;

  fn sub(self, target: Self) -> Self::Output {
    Vector3(self.0 - target.0, self.1 - target.1, self.2 - target.2)
  }
}

impl Mul<Flt> for Vector3 {
  type Output = Self;

  fn mul(self, s: Flt) -> Self::Output {
    Vector3(self.0 * s, self.1 * s, self.2 * s)
  }
}

impl Mul<Vector3> for Flt {
  type Output = Vector3;

  fn mul(self, vec: Vector3) -> Self::Output {
    Vector3(self * vec.0, self * vec.1, self * vec.2)
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
  fn norm(self) -> Flt;
  fn near(self, target: Self) -> bool;
}

pub trait Vector: BasicMatrix + Copy + std::marker::Sized {
  fn dot(self, target: Self) -> Flt;
  fn square(self) -> Flt {
    self.dot(self)
  }
  fn normalize(self) -> Option<Self>;
}

impl BasicMatrix for Vector3 {
  fn norm(self) -> Flt {
    f64::sqrt(self.square())
  }

  fn near(self, target: Self) -> bool {
    (self - target).norm() < NEARLY0
  }
}

impl Vector for Vector3 {
  fn dot(self, target: Self) -> Flt {
    self.0 * target.0 + self.1 * target.1 + self.2 * target.2
  }

  fn normalize(self) -> Option<Self> {
    let norm = self.norm();
    if norm == 0.0 {
      None
    } else {
      Some(self * (1.0 / norm))
    }
  }

}


/*






impl BasicMatrix for Vector3 {
  fn norm(self) -> f64 {
    f64::sqrt(self.square());
  }
 
  fn near(self, target: Self) -> bool {
    self.norm(self - target) < NEARLY0;
  }
}

impl Vector for Vector3 {
  fn inner(self, target: Vector3) -> f64 {
    self.0 * target.0 + self.1 * target.1 + self.2 * target.2;
  }

  fn normalize(self) -> Option<Self> {
    let len = self.square(self);
    if len == 0.0 {
      None;
    } else {
      Some(Vector3(self.0 / len, self.1 / len, self.2 / len));
    }
  }
}

impl Vector3 {
  pub fn outer(self, target: Vector3) -> Vector3 {
    Vector3(self.1 * target.2 - target.1 * self.2,
            self.2 * target.0 - target.2 * self.0,
            self.0 * target.1 - target.0 * self.1);
  }  
}

*/


