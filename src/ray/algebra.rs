// algebra module

//use std::f64;
use std::fmt;
use std::num::*;
//use std::intrinsics::const_eval_select;
//use std::num::ParseFloatError;
use std::ops::Neg;
use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;
use std::ops::Div;
use std::str::{FromStr};

use rand::Rng;
use regex::Regex;

use super::{Flt, NEARLY0, PI, PI2, PI_2ND};

pub trait BasicMatrix: Copy + Add + Sub + PartialEq {
  //fn div(&self, s: Flt) -> Option<Self>;
  fn norm(&self) -> Flt;
  fn length(&self) -> Flt {
    self.norm()
  }
  fn near(&self, target: &Self) -> bool;
}

pub trait Vector: BasicMatrix + std::marker::Sized {
  fn dot(&self, target: &Self) -> Flt;
  fn normalize(&self) -> Option<Self>;

  #[inline(always)]
  fn square(&self) -> Flt {
    self.dot(self)
  }
}

// 参考URLs
// https://scrapbox.io/nwtgck/Rustの演算子のオーバーロードで借用してmoveさせないようにする方法_-_std::ops::AddとかMulとか

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector3 {
  pub x: Flt,
  pub y: Flt,
  pub z: Flt
}

impl Vector3 {
  pub const O:  Vector3 = Vector3 {x: 0.0, y: 0.0, z: 0.0};
  pub const EX: Vector3 = Vector3 {x: 1.0, y: 0.0, z: 0.0};
  pub const EY: Vector3 = Vector3 {x: 0.0, y: 1.0, z: 0.0};
  pub const EZ: Vector3 = Vector3 {x: 0.0, y: 0.0, z: 1.0};

  #[inline(always)]
  pub fn new(x: Flt, y: Flt, z: Flt) -> Self {
    Vector3 {x, y, z}
  }

  #[inline(always)]
  pub fn cross(&self, target: &Self) -> Self {
    Vector3 {
      x: self.y * target.z - target.y * self.z,
      y: self.z * target.x - target.z * self.x,
      z: self.x * target.y - target.x * self.y
    }
  }

  #[inline(always)]
  pub fn x(&self) -> Flt {
    self.x
  }

  #[inline(always)]
  pub fn y(&self) -> Flt {
    self.y
  }

  #[inline(always)]
  pub fn z(&self) -> Flt {
    self.z
  }

  #[inline(always)]
  pub fn at(&self, k: usize) -> Flt {
    if k == 0 {
      self.x
    } else if k == 1 {
      self.y
    } else {
      self.z
    }
  }

}

impl fmt::Display for Vector3 {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "V3[{},{},{}]", self.x, self.y, self.z)
  }
}

// 文字列->構造体変換：参考URLs
// https://doc.rust-lang.org/std/str/trait.FromStr.html
// https://qiita.com/scivola/items/60141f262caa53983c3a
// https://text.baldanders.info/rust-lang/character-string-2/

#[derive(Debug, PartialEq)]
pub enum Vector3ParseError {
  InvalidFormat,
  ParseError(ParseFloatError),
}

impl FromStr for Vector3 {
  type Err = Vector3ParseError;

  #[inline]
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let re = Regex::new(r"^V3\[(\S+?),(\S+?),(\S+?)\]$").unwrap();
    let caps = re.captures(s);
    match caps {
      Some(cp) => {
        let x = cp[1].parse::<Flt>();
        if let Err(e) = x {
          return Err(Vector3ParseError::ParseError(e))
        }
        let y = cp[2].parse::<Flt>();
        if let Err(e) = y {
          return Err(Vector3ParseError::ParseError(e))
        }
        let z = cp[3].parse::<Flt>();
        if let Err(e) = z {
          return Err(Vector3ParseError::ParseError(e))
        }
        Ok(Vector3::new(x.unwrap(), y.unwrap(), z.unwrap()))
      },
      None => Err(Vector3ParseError::InvalidFormat),
    }
  }
}

impl Neg for Vector3 {
  type Output = Vector3;

  #[inline(always)]
  fn neg(self) -> Self::Output {
    Vector3 {x: -self.x, y: -self.y, z: -self.z}
  }
}

impl Add for Vector3 {
  type Output = Vector3;

  #[inline(always)]
  fn add(self, target: Self) -> Self::Output {
    Vector3 {
      x: self.x + target.x,
      y: self.y + target.y,
      z: self.z + target.z
    }
  }
}

impl Sub for Vector3 {
  type Output = Vector3;

  #[inline(always)]
  fn sub(self, target: Self) -> Self::Output {
    Vector3 {
      x: self.x - target.x,
      y: self.y - target.y,
      z: self.z - target.z
    }
  }
}

// vector x scaler
impl Mul<Flt> for Vector3 {
  type Output = Self;

  #[inline(always)]
  fn mul(self, s: Flt) -> Self::Output {
    Vector3 {
      x: self.x * s,
      y: self.y * s,
      z: self.z * s
    }
  }
}

// scaler x vector
impl Mul<Vector3> for Flt {
  type Output = Vector3;

  #[inline(always)]
  fn mul(self, v: Vector3) -> Self::Output {
    Vector3 {
      x: self * v.x,
      y: self * v.y,
      z: self * v.z
    }
  }
}

// vector x vector -> cross product (外積)
impl Mul<Vector3> for Vector3 {
  type Output = Self;

  #[inline(always)]
  fn mul(self, v: Self) -> Self::Output {
    self.cross(&v)
  }
}

impl Div<Flt> for Vector3 {
  type Output = Option<Vector3>;

  #[inline(always)]
  fn div(self, s: Flt) -> Option<Self> {
    if s == 0.0 {
      None
    } else {
      Some(Vector3::new(self.x / s, self.y / s, self.z / s))
    }
  }
}

impl BasicMatrix for Vector3 {
  #[inline(always)]
  fn norm(&self) -> Flt {
    f64::sqrt(self.square())
  }

  #[inline(always)]
  fn near(&self, target: &Self) -> bool {
    (*self - *target).norm() < NEARLY0
  }
}

impl Vector for Vector3 {
  #[inline(always)]
  fn dot(&self, target: &Self) -> Flt {
    self.x * target.x + self.y * target.y + self.z * target.z
  }

  #[inline(always)]
  fn normalize(&self) -> Option<Self> {
    let norm = self.norm();
    match norm {
      0.0 => None,
      _   => *self / norm,
    }
  }

}

impl From<Vector3> for [Flt; 3] {
  fn from(v: Vector3) -> Self {
    [v.x, v.y, v.z]
  }
}

#[test]
fn test_vector3() {

  /* Test for Vector3
   */
  // O
  assert_eq!(Vector3::O.x, 0.0);
  assert_eq!(Vector3::O.y, 0.0);
  assert_eq!(Vector3::O.z, 0.0);
  // EX
  assert_eq!(Vector3::EX.x, 1.0);
  assert_eq!(Vector3::EX.y, 0.0);
  assert_eq!(Vector3::EX.z, 0.0);
  // EY
  assert_eq!(Vector3::EY.x, 0.0);
  assert_eq!(Vector3::EY.y, 1.0);
  assert_eq!(Vector3::EY.z, 0.0);
  // EZ
  assert_eq!(Vector3::EZ.x, 0.0);
  assert_eq!(Vector3::EZ.y, 0.0);
  assert_eq!(Vector3::EZ.z, 1.0);
  // new
  let v1 = Vector3::new(1.0, 2.1, -3.4);
  assert_eq!(v1.x, 1.0);
  assert_eq!(v1.y, 2.1);
  assert_eq!(v1.z, -3.4);
  // x, y, z
  let v2 = Vector3::new(-2.0, 1.3, -4.8);
  assert_eq!(v2.x(), -2.0);
  assert_eq!(v2.y(), 1.3);
  assert_eq!(v2.z(), -4.8);
  // at
  assert_eq!(v2.at(0), -2.0);
  assert_eq!(v2.at(1), 1.3);
  assert_eq!(v2.at(2), -4.8);
  //cross
  assert_eq!(v1.cross(&v1), Vector3::new(0.0, 0.0, 0.0));
  let v3 = v1.cross(&v2);
  assert_eq!(v3.x, 2.1*(-4.8) - (-3.4)*1.3);
  assert_eq!(v3.y, (-3.4)*(-2.0) - 1.0*(-4.8));
  assert_eq!(v3.z, 1.0*1.3 - 2.1*(-2.0));
  let v3 = v2.cross(&v1);
  assert_eq!(v3.x, 1.3*(-3.4) - (-4.8)*2.1);
  assert_eq!(v3.y, (-4.8)*1.0 - (-3.4)*(-2.0));
  assert_eq!(v3.z, (-2.0)*2.1 - 1.3*1.0);
  assert_eq!(Vector3::EX.cross(&Vector3::EY), Vector3::EZ);
  assert_eq!(Vector3::EY.cross(&Vector3::EZ), Vector3::EX);
  assert_eq!(Vector3::EZ.cross(&Vector3::EX), Vector3::EY);

  /* Test for Display and FromStr
   */
  // fmt
  assert_eq!(format!("{}", v1), "V3[1,2.1,-3.4]");
  // from_str
  let v0 = "V3[-2,1.3,-4.8]".parse::<Vector3>().unwrap();
  assert_eq!(v0, v2);
  assert_eq!(format!("{}", v0).parse::<Vector3>().unwrap(), v0);
  let mut v00 = "v3[-2,1.3,-4.8]".parse::<Vector3>();
  assert_eq!(v00, Err(Vector3ParseError::InvalidFormat));
  v00 = "V3[-a,1.3,-4.8]".parse::<Vector3>();
  if let Err(_e) = v00 { assert!(true); };
  v00 = "V3[-2,1.-,-4.8]".parse::<Vector3>();
  if let Err(_e) = v00 { assert!(true); };
  v00 = "V3[-2,1.3,*4.8]".parse::<Vector3>();
  if let Err(_e) = v00 { assert!(true); };

  /* Test for Neg, Add, Sub, Mul, Div
   */
  // neg
  assert_eq!(v1.neg(), Vector3::new(-1.0, -2.1, 3.4));
  assert_eq!(v2.neg(), Vector3::new(2.0, -1.3, 4.8));
  // add
  assert_eq!(v1 + Vector3::O, v1);
  assert_eq!(v1 + v2, Vector3::new(1.0 + -2.0, 2.1 + 1.3, -3.4 + (-4.8)));
  assert_eq!(v2 + v1, Vector3::new(1.0 + -2.0, 2.1 + 1.3, -3.4 + (-4.8)));
  // sub
  assert_eq!(v1 - Vector3::O, v1);
  assert_eq!(v1 - v2, Vector3::new(1.0 - (-2.0), 2.1 - 1.3, -3.4 - (-4.8)));
  assert_eq!(v1 - v2.neg(), v1 + v2);
  // mul
  assert_eq!(v1 * 0.0, Vector3::O);
  assert_eq!(v1 * 1.0, v1);
  assert_eq!(v1 * 3.0, Vector3::new(1.0 * 3.0, 2.1 * 3.0, -3.4 * 3.0));
  assert_eq!(0.0 * v2, Vector3::O);
  assert_eq!(1.0 * v2, v2);
  assert_eq!(3.0 * v2, Vector3::new(3.0*(-2.0), 3.0*1.3, 3.0*(-4.8)));
  // v1:  1.0, 2.1, -3.4
  // v2: -2.0, 1.3, -4.8
  let r = v1.cross(&v2);
  assert_eq!(r, Vector3::new(2.1 * -4.8 - (-3.4) * 1.3, -3.4 * -2.0 - (-4.8) * 1.0, 1.0 * 1.3 - 2.1 * -2.0));
  assert_eq!(v1 * v2, r);
  // div
  assert_eq!(v1 / 1.0, Some(v1));
  assert_eq!(v1 / 3.0, Some(Vector3::new(1.0 / 3.0, 2.1 / 3.0, -3.4 / 3.0)));
  assert_eq!(v1 / 0.0, None);

  /* Test for BasicMatrix
   */
  // norm
  let len = f64::sqrt(1.0 * 1.0 + 2.1 * 2.1 + (-3.4) * (-3.4));
  assert_eq!(v1.norm(), len);
  // near
  assert_eq!(v1.near(&v1), true);
  let v4 = Vector3::new(1.0, 2.1, -3.4);
  assert_eq!(v1.near(&v4), true);
  let v4 = Vector3::new(1.00001, 2.10001, -3.40001);
  assert_eq!(v1.near(&v4), true);

  /* Test for Vector
   */
  // dot
  assert_eq!(v1.dot(&v1), len * len);
  assert_eq!(v1.dot(&v2), 1.0 * (-2.0) + 2.1 * 1.3 + (-3.4) * (-4.8));
  assert_eq!(Vector3::O.dot(&v1), 0.0);
  assert_eq!(Vector3::EX.dot(&Vector3::EY), 0.0);
  // square
  assert_eq!(Vector3::O.square(), 0.0);
  assert_eq!(Vector3::EX.square(), 1.0);
  assert_eq!(v1.square(), 1.0 * 1.0 + 2.1 * 2.1 + (-3.4) * (-3.4));
  // normalize
  assert_eq!(Vector3::EX.normalize(), Some(Vector3::new(1.0, 0.0, 0.0)));
  assert_eq!(v1.normalize(), Some(Vector3::new(1.0 / len, 2.1 / len, (-3.4) / len)));
  assert_eq!(Vector3::O.normalize(), None);

  /* Test for From
   */
  // from
  let a0: [Flt; 3] = v1.try_into().unwrap();
  assert_eq!(a0, [1.0, 2.1, -3.4]);



}



pub type Position3  = Vector3;

impl Position3 {
  #[inline(always)]
  pub fn init_pos(x: Flt, y: Flt, z: Flt) -> Self {
    Vector3 {x, y, z}
  }
}

#[test]
fn test_position3() {
  // init_pos
  let p1 = Position3::init_pos(2.1, 4.3, -5.2);
  assert_eq!(p1, Vector3::new(2.1, 4.3, -5.2));
  let p1 = Position3::init_pos(0.0, 0.0, 0.0);
  assert_eq!(p1, Vector3::O);
}

pub type Direction3 = Vector3;

impl Direction3 {
  #[inline(always)]
  pub fn init_dir(x: Flt, y: Flt, z: Flt) -> Option<Self> {
    let v = Vector3 {x, y, z};
    v.normalize()
  }

  // theta: angle with the Y-axis
  // phi  : anble aeound the Y-axis with the X-axis 
  #[inline(always)]
  pub fn init_dir_from_angle(theta: Flt, phi: Flt) -> Self {
    let sint = f64::sin(theta);
    let x = sint * f64::cos(phi);
    let y = f64::cos(theta);
    let z = sint * f64::sin(phi);
    Self::init_dir(x, y, z).unwrap()
  }
}

#[test]
fn test_direction3() {
  // init_dir
  let d1 = Direction3::init_dir(2.1, 4.3, -5.2);
  let d1len = f64::sqrt(2.1 * 2.1 + 4.3 * 4.3 + (-5.2) * (-5.2));
  assert_eq!(d1, Some(Vector3::new(2.1 / d1len, 4.3 / d1len, (-5.2) / d1len)));
  let d2 = Direction3::init_dir(0.0, 0.0, 0.0);
  assert_eq!(d2, None);
  // init_dir_fron_angle
  let d3 = Direction3::init_dir_from_angle(0.0, 0.0);
  assert_eq!(d3, Vector3::EY);
  let d3 = Direction3::init_dir_from_angle(PI_2ND, 0.0);
  assert_eq!(d3.near(&Vector3::EX), true);
  let d3 = Direction3::init_dir_from_angle(PI_2ND, PI_2ND);
  assert_eq!(d3.near(&Vector3::EZ), true);
}



// Vector2

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector2 {
  pub x: Flt,
  pub y: Flt
}

impl fmt::Display for Vector2 {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "V2[{},{}]", self.x, self.y)
  }
}

impl FromStr for Vector2 {
  type Err = std::num::ParseFloatError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let re = Regex::new(r"^V2\[(\S+?),(\S+?)\]$").unwrap();
    let caps = re.captures(s).unwrap();
    let x = caps[1].parse::<Flt>()?;
    let y = caps[2].parse::<Flt>()?;
    Ok(Vector2::new(x, y))
  }
}

impl Vector2 {
  pub const O:  Vector2 = Vector2 {x: 0.0, y: 0.0};
  pub const EX: Vector2 = Vector2 {x: 1.0, y: 0.0};
  pub const EY: Vector2 = Vector2 {x: 0.0, y: 1.0};

  #[inline(always)]
  pub fn new(x: Flt, y: Flt) -> Self {
    Vector2 {x, y}
  }
}

impl Add for Vector2 {
  type Output = Vector2;

  #[inline(always)]
  fn add(self, target: Self) -> Self::Output {
    Vector2 {
      x: self.x + target.x,
      y: self.y + target.y,
    }
  }
}

impl Sub for Vector2 {
  type Output = Vector2;

  #[inline(always)]
  fn sub(self, target: Self) -> Self::Output {
    Vector2 {
      x: self.x - target.x,
      y: self.y - target.y,
    }
  }
}

impl Div<Flt> for Vector2 {
  type Output = Option<Vector2>;

  #[inline(always)]
  fn div(self, s: Flt) -> Option<Self> {
    if s == 0.0 {
      None
    } else {
      Some(Vector2 {x: self.x / s, y: self.y / s})
    }
  }
}

impl BasicMatrix for Vector2 {
  #[inline(always)]
  fn norm(&self) -> Flt {
    f64::sqrt(self.square())
  }

  #[inline(always)]
  fn near(&self, target: &Self) -> bool {
    (*self - *target).norm() < NEARLY0
  }
}

impl Vector for Vector2 {
  #[inline(always)]
  fn dot(&self, target: &Self) -> Flt {
    self.x * target.x + self.y * target.y
  }

  #[inline(always)]
  fn normalize(&self) -> Option<Self> {
    let norm = self.norm();
    match norm {
      0.0 => None,
      _   => *self / norm,
    }
  }

}

#[test]
fn test_vector2() {
  // O
  assert_eq!(Vector2::O.x, 0.0);
  assert_eq!(Vector2::O.y, 0.0);
  // EX
  assert_eq!(Vector2::EX.x, 1.0);
  assert_eq!(Vector2::EX.y, 0.0);
  // EY
  assert_eq!(Vector2::EY.x, 0.0);
  assert_eq!(Vector2::EY.y, 1.0);

  // new
  let v0 = Vector2::new(1.0, 2.1);
  assert_eq!(v0.x, 1.0);
  assert_eq!(v0.y, 2.1);
  assert_eq!(format!("{}", v0), "V2[1,2.1]");

  let v1 = Vector2::new(1.0, 2.1);
  let v2 = Vector2::new(-2.0, 1.3);

  // add
  assert_eq!(v1 + Vector2::O, v1);
  assert_eq!(v1 + v2, Vector2::new(1.0 + -2.0, 2.1 + 1.3));
  assert_eq!(v2 + v1, Vector2::new(1.0 + -2.0, 2.1 + 1.3));
  // sub
  assert_eq!(v1 - Vector2::O, v1);
  assert_eq!(v1 - v2, Vector2::new(1.0 - (-2.0), 2.1 - 1.3));
  // div
  assert_eq!(v1 / 1.0, Some(v1));
  assert_eq!(v1 / 3.0, Some(Vector2::new(1.0 / 3.0, 2.1 / 3.0)));
  assert_eq!(v1 / 0.0, None);

  /* Test for BasicMatrix
   */
  // norm
  let len = f64::sqrt(1.0 * 1.0 + 2.1 * 2.1);
  assert_eq!(v1.norm(), len);
  // near
  assert_eq!(v1.near(&v1), true);
  let v4 = Vector2::new(1.0, 2.1);
  assert_eq!(v1.near(&v4), true);
  let v4 = Vector2::new(1.00001, 2.10001);
  assert_eq!(v1.near(&v4), true);

  /* Test for Vector
   */
  // dot
  assert_eq!(v1.dot(&v1), len * len);
  assert_eq!(v1.dot(&v2), 1.0 * (-2.0) + 2.1 * 1.3);
  assert_eq!(Vector2::O.dot(&v1), 0.0);
  assert_eq!(Vector2::EX.dot(&Vector2::EY), 0.0);
  // square
  assert_eq!(Vector2::O.square(), 0.0);
  assert_eq!(Vector2::EX.square(), 1.0);
  assert_eq!(v1.square(), 1.0 * 1.0 + 2.1 * 2.1);
  // normalize
  assert_eq!(Vector2::EX.normalize(), Some(Vector2::new(1.0, 0.0)));
  assert_eq!(v1.normalize(), Some(Vector2::new(1.0 / len, 2.1 / len)));
  assert_eq!(Vector2::O.normalize(), None);

}


// ---------------------
// Ray

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Ray {
  pub pos: Position3,
  pub dir: Direction3,
}

impl Ray {
  pub fn new(p: &Position3, d: &Direction3) -> Option<Ray> {
    match d.normalize() {
      Some(d2) => Some(Ray {pos: *p, dir: d2}),
      None => None
    }
  }

  pub fn new_from_elem(px: Flt, py: Flt, pz: Flt, dx: Flt, dy: Flt, dz: Flt) -> Option<Ray> {
    let p = Position3::init_pos(px, py, pz);
    let d = Direction3::init_dir(dx, dy, dz);
    match d {
      Some(dir) => Self::new(&p, &dir),
      None               => None,
    }
  }

  pub fn target(&self, t: Flt) -> Position3 {
    self.pos + self.dir * t
  }
}

impl fmt::Display for Ray {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "RAY[{},{}]", self.pos, self.dir)
  }
}

#[derive(Debug, PartialEq)]
pub enum RayParseError {
  InvalidFormat,
  ParseError(Vector3ParseError),
}

impl FromStr for Ray {
  type Err = RayParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let re = Regex::new(r"^RAY\[(V3\[\S+\]),(V3\[\S+\])\]$").unwrap();
    let caps = re.captures(s);
    match caps {
      Some(cp) => {
        let p = cp[1].parse::<Vector3>();
        if let Err(e) = p {
          return Err(RayParseError::ParseError(e))
        };
        let d = cp[2].parse::<Vector3>();
        if let Err(e) = d {
          return Err(RayParseError::ParseError(e))
        };
        Ok(Ray::new(&p.unwrap(), &d.unwrap()).unwrap())
      },
      None => Err(RayParseError::InvalidFormat),
    }
  }
}

pub type SurfacePoint = Ray;

#[test]
fn test_ray() {
  let p1 = Position3::init_pos(1.0, 2.0, 3.0);
  let d1 = Direction3::init_dir(1.0, 1.0, 1.0);
  let r1 = Ray::new(&p1, &(d1.unwrap())).unwrap();
  assert_eq!(r1, Ray::new(&Vector3::new(1.0, 2.0, 3.0), &Vector3::new(0.5773502691896258, 0.5773502691896258, 0.5773502691896258)).unwrap());
  assert_eq!(format!("{}", r1), "RAY[V3[1,2,3],V3[0.5773502691896258,0.5773502691896258,0.5773502691896258]]");
  let r2 = Ray::from_str("RAY[V3[0.5,1.2,0.1],V3[-1.0,1.0,2.5]]");
  let r22 = r2.unwrap();
  assert_eq!(r22, Ray {pos: Position3::init_pos(0.5, 1.2, 0.1), dir: Direction3::init_dir(-1.0, 1.0, 2.5).unwrap()});
  // target
  let s1 = f64::sqrt(1.0 / 3.0) * 2.0;
  assert_eq!(r1.target(2.0), Vector3::new(1.0 + s1, 2.0 + s1, 3.0 + s1));
  let s2 = -1.2 / f64::sqrt(1.0 + 1.0 + 2.5 * 2.5);
  assert_eq!(r22.target(-1.2), Vector3::new(0.5 + (-1.0) * s2, 1.2 + 1.0 * s2, 0.1 + 2.5 * s2));
}


// SUPPORT FUNCTIONS

/* ランダムなベクトルの生成（4種）
 */

// ランダムな２つの角度で生成する方法
pub fn generate_random_dir1() -> Direction3 {
  let mut rng = rand::thread_rng();
  let theta: Flt = rng.gen_range(0.0, PI);
  let phi  : Flt = rng.gen_range(0.0, PI2);
  Direction3::init_dir_from_angle(theta, phi)
}

// ３要素が半径1.0の球内にあるかどうかで決める方法
pub fn generate_random_dir2() -> Direction3 {
  let mut rng = rand::thread_rng();
  loop {
    let x: Flt = rng.gen_range(-1.0, 1.0);
    let y: Flt = rng.gen_range(-1.0, 1.0);
    let z: Flt = rng.gen_range(-1.0, 1.0);
    let v = Vector3 {x, y, z};
    let len = v.norm();
    if 0.0 < len && len <= 1.0  {
      return v.normalize().unwrap();
    }
  }
}

// generate_random_dir3 はdir2と同じのため廃止

// 平面角度φと上下角度ξから算出
pub fn generate_random_dir4() -> Direction3 {
  let mut rng = rand::thread_rng();
  let phi: Flt = rng.gen_range(0.0, PI2);
  let xi1: Flt = rng.gen_range(-1.0, 1.0);
  let xi2: Flt = f64::sqrt(1.0 - xi1.powf(2.0));
  let x: Flt = xi2 * f64::cos(phi);
  //let y: Flt = xi1;
  let z: Flt = xi2 * f64::sin(phi);
  Direction3::init_dir(x, xi1, z).unwrap()
}

// blurred_vector
//   与えられた方向ベクトル(nvec)を中心としてブレの大きさ(pow)に応じて
//   ランダムな方向ベクトルを生成する
//   pow=1のとき、generate_random_dir4と同じ
//   pow=♾️のとき、ブレは限りなくゼロとなり、nvecと同じベクトルとなる
//   一様乱数 xi[0,1]のとき、yがcos(xi)ではなくxiの冪乗なのは、緯度が大きいほど半球の表面積が
//   小さくなり密度が上がるため。
/*
  http://www.raytracegroundup.com/downloads/Chapter25.pdf
  https://cg.informatik.uni-freiburg.de/course_notes/graphics2_08_renderingEquation.pdf
  https://graphics.cg.uni-saarland.de/courses/ris-2018/slides/09_BRDF_LightSampling.pdf
  x = cos(2 pi xi2) sqrt(1 - xi1^(2/(n+1)))
  y = xi1^(1/(n+1))
  z = sin(2 pi xi2) sqrt(1 - xi1^(2/(n+1)))
  ※ pow = 1/(n+1) の場合
*/
pub fn blurred_vector(nvec: &Direction3, pow: Flt) -> Direction3 {
  let mut rng = rand::thread_rng();
  let phi: Flt = rng.gen_range(0.0, PI2);
  let xi1: Flt = rng.gen_range(0.0, 1.0);
  let uvec0 = nvec.cross(&Vector3::new(0.00424, 1.0, 0.00764)).normalize();
  let uvec = match uvec0 {
    Some(v) => v,
    None    => nvec.cross(&Vector3::new(1.0, 0.00424, 0.00764))
  };
  let vvec = uvec.cross(nvec);
  let xi1_1 = xi1.powf(pow);
  let rt = f64::sqrt(1.0 - xi1_1 * xi1_1);

  let nvec2 = (f64::cos(phi) * rt) * uvec
            + xi1_1 * *nvec
            + (f64::sin(phi) * rt) * vvec;
  match nvec2.normalize() {
    Some(v) => v,
    None             => Vector3::EX
  }
}

