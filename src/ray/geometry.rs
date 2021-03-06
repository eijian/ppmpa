// geometry

use core::num::ParseFloatError;
use std::f64;
use std::fmt;
use std::str::*;

use regex::Regex;

use super::*;
use super::algebra::*;


// ---------------------
// Ray

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Ray {
  pub pos: Position3,
  pub dir: Direction3,
}

impl fmt::Display for Ray {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "RAY[{},{}]", self.pos, self.dir)
  }
}

impl FromStr for Ray {
  type Err = ParseFloatError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let re = Regex::new(r"^RAY\[(V3\[\S+\]),(V3\[\S+\])\]$").unwrap();
    let caps = re.captures(s).unwrap();
    let p = caps[1].parse::<Vector3>()?;
    let d = caps[2].parse::<Vector3>()?;
    Ok(Ray::new(&p, &d))
  }
}

impl Ray {
  pub fn new(p: &Position3, d: &Direction3) -> Ray {
    Ray {pos: *p, dir: *d}
  }

  pub fn new_from_elem(px: Flt, py: Flt, pz: Flt, dx: Flt, dy: Flt, dz: Flt) -> Option<Ray> {
    let p = Position3::new_pos(px, py, pz);
    let d = Direction3::new_dir(dx, dy, dz);
    if d == None {
      None
    } else {
      Some(Self::new(&p, &(d.unwrap())))
    }
  }

  pub fn target(&self, t: Flt) -> Position3 {
    self.pos + self.dir * t
  }
}


// ---------------------
// shape

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Shape {
  Point {
    position: Position3,
  },
  Plain {
    nvec: Direction3,
    dist: Flt,
  },
  Sphere {
    center: Position3,
    radius: Flt,
  },
  Polygon {
    position: Position3,
    nvec: Direction3,
    dir1: Direction3,
    dir2: Direction3,
  },
  Parallelogram {
    position: Position3,
    nvec: Direction3,
    dir1: Direction3,
    dir2: Direction3,    
  },
}

impl Shape {
  pub fn new_polygon(p0: &Position3, p1: &Position3, p2: &Position3) -> Shape {
    let d1 = *p1 - *p0;
    let d2 = *p2 - *p0;
    let n = d1.cross(&d2).normalize().unwrap();
    Shape::Polygon {
      position: *p0,
      nvec: n,
      dir1: d1,
      dir2: d2
    }
  }

  pub fn new_parallelogram(p0: &Position3, p1: &Position3, p2: &Position3) -> Shape {
    let d1 = *p1 - *p0;
    let d2 = *p2 - *p0;
    let n = d1.cross(&d2).normalize().unwrap();
    Shape::Parallelogram {
      position: *p0,
      nvec: n,
      dir1: d1,
      dir2: d2
    }
  }

  pub fn get_normal(&self, p: &Position3) -> Option<Direction3> {
    match self {
      Shape::Point {position: _}
        => None,
      Shape::Plain {nvec, dist: _}
        => Some(*nvec),
      Shape::Sphere {center, radius:_}
        => (*p - *center).normalize(),
      Shape::Polygon {position: _, nvec, dir1: _, dir2: _}
        => Some(*nvec),
      Shape::Parallelogram {position: _, nvec, dir1: _, dir2: _}
        => Some(*nvec),
    }
  }

  pub fn distance(&self, r: &Ray) -> Vec<Flt> {
    match self {
      Shape::Point {position: _}
        => vec![],
      Shape::Plain {nvec, dist}
        => distance_plain(r, nvec, dist),
      Shape::Sphere {center, radius}
        => distance_sphere(r, center, radius),
      Shape::Polygon {position, nvec: _, dir1, dir2}
        => distance_polygon(&1.0, r, position, dir1, dir2),
      Shape::Parallelogram {position, nvec: _, dir1, dir2}
        => distance_polygon(&2.0, r, position, dir1, dir2),
    }
  }

}

pub fn method_moller(l: &Flt, p0: &Position3, d1: &Direction3, d2: &Direction3, p: &Position3, d: &Direction3) -> Option<(Flt, Flt, Flt)> {
  let re2 = d.cross(d2);
  let det_a = re2.dot(d1);
  let pp = *p - *p0;
  let te1 = pp.cross(d1);
  let u = re2.dot(&pp) / det_a;
  let v = te1.dot(d)  / det_a;
  let t = te1.dot(d2) / det_a;
  if det_a == 0.0 ||
     u < 0.0 || u > 1.0 ||
     v < 0.0 || v > 1.0 ||
     u + v > *l {
    None
  } else {
    Some((u, v, t))
  }
}


// utility functions

fn distance_plain(r: &Ray, n: &Direction3, d: &Flt) -> Vec<Flt> {
  let cos0 = n.dot(&r.dir);
  if cos0 == 0.0 {
    vec![]
  } else {
    vec![(*d + n.dot(&r.pos)) / -cos0]
  }
}

fn distance_sphere(r: &Ray, c: &Position3, rad: &Flt) -> Vec<Flt> {
  let o = *c - r.pos;
  let t0 = o.dot(&r.dir);
  let t1 = rad * rad - (o.square() - (t0 * t0));
  let t2 = f64::sqrt(t1);
  if t1 <= 0.0 {
    vec![]
  } else {
    if t2 == 0.0 {
      vec![t0]
    } else {
      vec![t0 - t2, t0 + t2]
    }
  }
}

fn distance_polygon(l: &Flt, r: &Ray, p: &Position3, d1: &Direction3, d2: &Direction3) -> Vec<Flt> {
  let res = method_moller(l, p, d1, d2, &r.pos, &r.dir);
  if res == None {
    vec![]
  } else {
    vec![res.unwrap().2]
  }
}


//----
// tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_ray() {
    let p1 = Position3::new_pos(1.0, 2.0, 3.0);
    let d1 = Direction3::new_dir(1.0, 1.0, 1.0);
    let r1 = Ray::new(&p1, &(d1.unwrap()));
    assert_eq!(r1, Ray::new(&Vector3::new(1.0, 2.0, 3.0), &Vector3::new(0.5773502691896258, 0.5773502691896258, 0.5773502691896258)));
    assert_eq!(format!("{}", r1), "RAY[V3[1,2,3],V3[0.5773502691896258,0.5773502691896258,0.5773502691896258]]");
    let r2 = Ray::from_str("RAY[V3[0.5,1.2,0.1],V3[-1.0,1.0,2.5]]");
    let r22 = r2.unwrap();
    assert_eq!(r22, Ray {pos: Vector3 {v: [0.5, 1.2, 0.1]}, dir: Vector3 {v: [-1.0, 1.0, 2.5]}});
  }

  #[test]
  fn test_target() {
    let r = Ray::new_from_elem(1.0, 1.0, 1.0, -1.0, -1.0, -1.0).unwrap();
    assert_eq!(r.target(2.0), Vector3::new(-0.15470053837925168, -0.15470053837925168, -0.15470053837925168));
  }

  #[test]
  fn test_getnormal() {
    let pl = Shape::Plain {nvec: Vector3::EY, dist: 1.0};
    assert_eq!(pl.get_normal(&Vector3::new(0.0, 1.0, 0.0)), Some(Vector3::new(0.0, 1.0, 0.0)));
    let sp = Shape::Sphere {center: Vector3::O, radius: 2.0};
    assert_eq!(sp.get_normal(&Vector3::new(2.0, 0.0, 0.0)), Some(Vector3::new(1.0, 0.0, 0.0)));
    let p = Vector3::O;
    let p1 = Vector3::new(2.0, 1.0, 0.0);
    let p2 = Vector3::new(0.0, 1.0, 2.0);
    let po = Shape::new_polygon(&p, &p1, &p2);
    assert_eq!(po.get_normal(&Vector3::new(0.0, 1.0, 0.0)), Some(Vector3::new(0.4082482904638631, -0.8164965809277261, 0.4082482904638631)));
    let pa = Shape::new_parallelogram(&p, &p1, &p2);
    assert_eq!(pa.get_normal(&Vector3::new(0.0, 1.0, 0.0)), Some(Vector3::new(0.4082482904638631, -0.8164965809277261, 0.4082482904638631)));
  }


}



