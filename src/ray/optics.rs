// optics

use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;

use super::*;
use super::algebra::*;
use super::geometry::*;
use super::physics::*;

#[derive(Debug, PartialEq)]
pub enum PhotonFilter {
  Non,
  Cone,
  Gauss,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Radiance(Flt, Flt, Flt);

impl Add for Radiance {
  type Output = Self;

  fn add(self, target: Self) -> Self::Output {
    Radiance(self.0 + target.0, self.1 + target.1, self.2 + target.2)
  }
}

impl Sub for Radiance {
  type Output = Self;

  fn sub(self, target: Self) -> Self::Output {
    Radiance(self.0 - target.0, self.1 - target.1, self.2 - target.2)
  }
}

impl Mul<Flt> for Radiance {
  type Output = Self;

  fn mul(self, s: Flt) -> Self::Output {
    Radiance(self.0 * s, self.1 * s, self.2 * s)
  }
}

impl BasicMatrix for Radiance {
  fn norm(&self) -> Flt {
    //println!("r:{},g:{},b:{}", self.0, self.1, self.2);
    rabs(self.0) + rabs(self.1) + rabs(self.2)
  }

  fn near(&self, target: &Self) -> bool {
    (*self - *target).norm() < NEARLY0
  }
}

impl Mul<Radiance> for Color {
  type Output = Radiance;

  fn mul(self, r: Radiance) -> Self::Output {
    Radiance(self.0 * r.0, self.1 * r.1, self.2 * r.2)
  }
}

impl Radiance {
  pub const RADIANCE0: Radiance = Radiance(0.0, 0.0, 0.0);
  pub const RADIANCE1: Radiance = Radiance(1.0, 1.0, 1.0);

  pub fn select_wavelength(&self, w: Wavelength) -> Flt {
    match w {
      Wavelength::Red   => self.0,
      Wavelength::Green => self.1,
      Wavelength::Blue  => self.2,
    }
  }
}


// support functions 

fn rabs(d: Flt) -> Flt {
  if d < 0.0 {
    -d
  } else {
    d
  }
}

// --------------------------
// photon
// --------------------------

#[derive(Debug)]
pub struct Photon(pub Wavelength, pub Ray);

impl Photon {
  pub fn new(wl: &Wavelength, r: &Ray) -> Self {
    Photon(*wl, *r)
  }
}

pub type PhotonCache = Photon;

impl PhotonCache {
  pub fn to_info(&self) -> PhotonInfo {
    PhotonInfo(self.0, self.1.0, -self.1.1)
  }
}

pub fn square_distance(pi1: &PhotonInfo, pi2: &PhotonInfo) -> Flt {
  let d = pi1.1 - pi2.1;
  d.square()
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PhotonInfo(pub Wavelength, pub Position3, pub Direction3);

pub struct PhotonMap {
  power:    Flt,
  nearest:  fn(PhotonInfo) -> Vec<PhotonInfo>,
  inradius: fn(PhotonInfo) -> Vec<PhotonInfo>,
}

impl PhotonInfo {
  pub fn dummy(p: &Position3) -> PhotonInfo {
    PhotonInfo(Wavelength::Red, *p, Vector3::EX)
  }

  pub fn pos(&self) -> Position3 {
    self.1
  }
  pub fn dir(&self) -> Direction3 {
    self.2
  }

  pub fn to_radiance(&self, n: &Direction3, pw: &Flt) -> Radiance {
    let cos0 = n.dot(&self.2);
    let pw2 = if cos0 > 0.0 { pw * cos0 } else { 0.0 };
    match self.0 {
      Wavelength::Red   => Radiance(pw2, 0.0, 0.0),
      Wavelength::Green => Radiance(0.0, pw2, 0.0),
      Wavelength::Blue  => Radiance(0.0, 0.0, pw2),
    }
  }
}


//
// tests
//

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_pf() {
    let pf1 = PhotonFilter::Gauss;
    assert_eq!(pf1, PhotonFilter::Gauss);
  }

  #[test]
  fn test_radiance() {
    let r1 = Radiance::RADIANCE1;
    assert_eq!(r1, Radiance(1.0, 1.0, 1.0));
    let r2 = Radiance(1.0, 0.8, 0.6);
    assert_eq!(r1 + r2, Radiance(2.0, 1.8, 1.6));
    assert_eq!((r1 - r2).near(&Radiance(0.0, 0.2, 0.4)), true);
    assert_eq!(r2.norm(), 1.0 + 0.8 + 0.6);
    assert_eq!((r2 - r1).norm(), 0.6);
    assert_eq!(r2 * 2.5, Radiance(2.5, 2.0, 1.5));
    let c = Color(1.0, 2.0, 4.0);
    assert_eq!(c * r2, Radiance(1.0, 1.6, 2.4));
    assert_eq!(r2.select_wavelength(Wavelength::Red), 1.0);
    assert_eq!(r2.select_wavelength(Wavelength::Green), 0.8);
    assert_eq!(r2.select_wavelength(Wavelength::Blue), 0.6);
  }

  #[test]
  fn test_rbs() {
    assert_eq!(rabs(0.0), 0.0);
    assert_eq!(rabs(1.1), 1.1);
    assert_eq!(rabs(-1.1), 1.1);
  }


}

