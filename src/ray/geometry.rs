// geometry

//use std::fmt;
use std::fmt::Debug;
//use std::path::Component::Normal;
//use std::str::*;

use rand::Rng;
//use regex::Regex;

use super::{Flt, NEARLY0, PI4};
use super::algebra::{BasicMatrix, Direction3, Position3, Ray, Vector, Vector2, Vector3,
                     generate_random_dir1, generate_random_dir2};

// -----------------------
// ShapeFunc Trait

pub trait ShapeFunc: Debug + PartialEq + Clone + Copy {
  fn get_normal(&self, pos: &Position3)  -> Option<Direction3>;
  fn distance(&self, ray: &Ray) -> Option<(Vector2, Flt, Option<usize>)>;
  fn n_surface(&self) -> usize;
  fn surface_area(&self) -> Flt;
  fn random_point(&self) -> Ray;
  fn get_direction(&self, destpos: &Position3, srcpos: &Position3) -> Option<Direction3> {
    let nvec = self.get_normal(destpos);
    if nvec == None {
      return None
    }
    let dir  = *destpos - *srcpos;
    let cos  = nvec.unwrap().dot(&dir);
    if cos > 0.0 {
      None
    } else {
      Some(dir)
    }
  }
}

// Shape --------------------------------------------------

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Shape {
  Point {
    pos: Position3,
  },
  Plane {
    nvec: Direction3,
    dist: Flt,
  },
  Sphere {
    center:  Position3,
    radius:  Flt,
    radius2: Flt,
  },
  Triangle {
    pos : Position3,
    nvec: Direction3,
    dir1: Direction3,
    dir2: Direction3,
  },
  Parallelogram {
    pos : Position3,
    nvec: Direction3,
    dir1: Direction3,
    dir2: Direction3,
  },
}

impl Shape {

  // Point generator -----------
  #[inline(always)]
  pub fn new_point(x: Flt, y: Flt, z: Flt) -> Self {
    Shape::Point {pos: Position3::init_pos(x, y, z)}
  }

  #[inline(always)]
  pub fn new_point_from_position(pos: &Position3) -> Self {
    Self::new_point(pos.x, pos.y, pos.z)
  }

  // Plane generator -----------
  // nvec . X + dist = 0
  #[inline(always)]
  pub fn new_plane(nvec: &Direction3, dist: Flt) -> Self {
    Shape::Plane {nvec: *nvec, dist}
  }
  // ax + by + cz + d = 0
  #[inline(always)]
  pub fn new_plane_from_coefficient(a: Flt, b: Flt, c: Flt, d:Flt) -> Self {
    let n0 = Vector3::new(a, b, c);
    let len = n0.length();
    let dist = -d / len;
    Shape::Plane {nvec: n0.normalize().unwrap(), dist}
  }
  // nvec/|nvec| . pos + dist = 0 -> dist = -(nvec/|nvec| . pos)
  #[inline(always)]
  pub fn new_plane_from_dir_and_pos(nvec: &Direction3, pos: &Position3) -> Self {
    let nvec2 = nvec.normalize().unwrap();
    let dist = -(nvec2.dot(pos));
    Shape::Plane {nvec: nvec2, dist}
  }

  // Sphere generator ------------
  #[inline(always)]
  pub fn new_sphere(center: &Position3, radius: Flt) -> Self {
    Shape::Sphere {center: *center, radius, radius2: radius * radius}
  }

  // Triangle generator ----------
  #[inline(always)]  
  pub fn new_triangle(pos: &Position3, nvec: &Direction3, dir1: &Direction3, dir2: &Direction3) -> Self {
    Shape::Triangle {pos: *pos, nvec: *nvec, dir1: *dir1, dir2: *dir2}
  }

  #[inline(always)]
  pub fn new_triangle_with_points(pos0: &Position3, pos1: &Position3, pos2: &Position3) -> Self {
    let dir1 = *pos1 - *pos0;
    let dir2 = *pos2 - *pos0;
    let nvec = dir1.cross(&dir2).normalize().unwrap();
    Shape::Triangle {pos: *pos0, nvec, dir1, dir2}
  }

  #[inline(always)]
  pub fn new_triangle_with_normal(pos0: &Position3, pos1: &Position3, pos2: &Position3, nvec: &Direction3) -> Self {
    let dir1 = *pos1 - *pos0;
    let dir2 = *pos2 - *pos0;
    Shape::Triangle {pos: *pos0, nvec: nvec.normalize().unwrap(), dir1, dir2}
  }

  // Parallelogram (平行四辺形) ------
  #[inline(always)]
  pub fn new_parallelogram(pos: &Position3, nvec: &Direction3, dir1: &Direction3, dir2: &Direction3) -> Self {
    Shape::Parallelogram {pos: *pos, nvec: *nvec, dir1: *dir1, dir2: *dir2}
  }

  #[inline(always)]
  pub fn new_parallelogram_with_points(pos0: &Position3, pos1: &Position3, pos2: &Position3) -> Self {
    let dir1 = *pos1 - *pos0;
    let dir2 = *pos2 - *pos0;
    let nvec = dir1.cross(&dir2).normalize().unwrap();
    Shape::Parallelogram {pos: *pos0, nvec, dir1, dir2}
  }

  #[inline(always)]
  pub fn new_parallelogram_with_normal(pos0: &Position3, pos1: &Position3, pos2: &Position3, nvec: &Direction3) -> Self {
    let dir1 = *pos1 - *pos0;
    let dir2 = *pos2 - *pos0;
    Shape::Parallelogram {pos: *pos0, nvec: nvec.normalize().unwrap(), dir1, dir2}
  }

}

impl ShapeFunc for Shape {

  fn get_normal(&self, pos: &Position3)  -> Option<Direction3> {
    match self {
      Shape::Point {..} => None,
      Shape::Plane {nvec, ..} => Some(*nvec),
      Shape::Sphere {center, ..}=> (*pos - *center).normalize(),
      Shape::Triangle {nvec, ..} => Some(*nvec),
      Shape::Parallelogram {nvec, ..} => Some(*nvec),
    }
  }

  fn distance(&self, ray: &Ray) -> Option<(Vector2, Flt, Option<usize>)> {
    match self {
      Shape::Point {..} => None,
      Shape::Plane {nvec, dist} => {
        let cos = nvec.dot(&ray.dir);
        let dist = (dist + nvec.dot(&ray.pos)) / -cos;
        if cos == 0.0 {
          None
        } else if dist < NEARLY0 {
          None
        } else {
          Some((Vector2::O, dist, None))
        }
      },
      Shape::Sphere {center, radius: _, radius2} => {
        let o = *center - ray.pos;
        let t0 = o.dot(&ray.dir);
        let t1 = radius2 - (o.square() - (t0 * t0));

        if t1 <= 0.0 {
          return None
        };

        let t2 = f64::sqrt(t1);
        if t2 == 0.0 {
          if t0 >= NEARLY0 {
            return Some((Vector2::O, t0, None))
          } else {
            return None
          }
        };

        if t0 - t2 >= NEARLY0 {
          Some((Vector2::O, t0 - t2, None))
        } else if t0 + t2 >= NEARLY0 {
          Some((Vector2::O, t0 + t2, None))
        } else {
          None
        }
      },
      Shape::Triangle {pos, nvec: _, dir1, dir2} => {
        match method_moller(1.0, pos, dir1, dir2, ray) {
          None => None,
          Some((uv, t)) => if t < NEARLY0 {
            None
          } else {
            Some((uv, t, None))
          },
        }
      },
      Shape::Parallelogram {pos, nvec: _, dir1, dir2}=> {
        match method_moller(2.0, pos, dir1, dir2, ray) {
          None => None,
          Some((uv, t)) => if t < NEARLY0 {
            None
          } else {
            Some((uv, t, None))
          },
        }
      },
    }
  }

  fn n_surface(&self) -> usize {
    match self {
      Shape::Point {..} => 0,
      Shape::Plane {..} => 1,
      Shape::Sphere {..} => 1,
      Shape::Triangle {..} => 1,
      Shape::Parallelogram {..} => 1,
    }
  }


  fn surface_area(&self) -> Flt {
    match self {
      Shape::Point {..} => 0.0,
      Shape::Plane {..} => f64::INFINITY,
      Shape::Sphere {center: _, radius: _, radius2} => PI4 * radius2,
      Shape::Triangle {nvec: _, pos: _, dir1, dir2} => dir1.cross(dir2).norm() / 2.0,
      Shape::Parallelogram {nvec: _, pos: _, dir1, dir2} => dir1.cross(dir2).norm(),
    }
  }

  fn random_point(&self) -> Ray {
    match self {
      Shape::Point {pos} => Ray::new(pos, &generate_random_dir1()).unwrap(),
      Shape::Plane {nvec, ..} => Ray::new(&Vector3::O, nvec).unwrap(),
      Shape::Sphere {center, radius, radius2: _} => {
        let nvec = generate_random_dir2();
        let pos = *center + (*radius * nvec);
        Ray::new(&pos, &nvec).unwrap()
      },
      Shape::Triangle {pos, nvec, dir1, dir2} => {
        let mut rng = rand::thread_rng();
        let m: Flt = rng.gen_range(0.0, 1.0);
        let n: Flt = rng.gen_range(0.0, 1.0);
        let (m2, n2) = if m + n > 1.0 {
          if m > n {
            (1.0 - m, n)
          } else {
            (m, 1.0 - n)
          }
        } else {
          (m, n)
        };
        Ray {pos: *pos + m2 * *dir1 + n2 * *dir2, dir: *nvec}
      },
      Shape::Parallelogram {pos, nvec, dir1, dir2} => {
        let mut rng = rand::thread_rng();
        let m: Flt = rng.gen_range(0.0, 1.0);
        let n: Flt = rng.gen_range(0.0, 1.0);
        Ray {pos: *pos + m * *dir1 + n * *dir2, dir: *nvec}
      },
    }
  }

}

#[test]
fn test_point() {
  // new, new_from_position
  let p1 = Shape::new_point(1.0, 2.1, 3.2);
  assert_eq!(p1, Shape::Point {pos: Vector3 {x: 1.0, y: 2.1, z: 3.2}});
  let pos= Vector3::new(1.0, 2.1, 3.2);
  let p2 = Shape::new_point_from_position(&pos);
  // get_normal
  assert_eq!(p1.get_normal(&Vector3::O), None);
  assert_eq!(p2.get_normal(&Vector3::new(1.0, 2.1, 3.2)), None);
  // distance
  let ray = Ray::new(&Vector3::O, &Vector3::EX).unwrap();
  assert_eq!(p1.distance(&ray), None);
  // n_surface
  assert_eq!(p1.n_surface(), 0);
  // surface_area
  assert_eq!(p1.surface_area(), 0.0);
  assert_eq!(p2.surface_area(), 0.0);
  // random_point
  let rr1 = p1.random_point();
  if let Shape::Point {pos} = p1 {
    assert_eq!(rr1.pos, pos);
  };
  assert!((rr1.dir.length() - 1.0).abs() < NEARLY0);
}

#[test]
fn test_plane() {
  // new, new_from_coefficient, new_from_dir_and_pos
  let n = Vector3::new(1.0, 2.0, 3.0);
  let len = n.norm();
  let nvec = n.normalize().unwrap();
  let p = Position3::new(0.0, 1.0, 0.0);
  let dist = -2.0 / len;
  let p1 = Shape::new_plane(&nvec, dist);
  let p2 = Shape::new_plane_from_coefficient(1.0, 2.0, 3.0, 2.0);
  let p3 = Shape::new_plane_from_dir_and_pos(&n, &p);
  assert_eq!(p1, p2);
  assert_eq!(p1, p3);
  assert_eq!(p2, p3);
  // get_normal
  assert_eq!(p2.get_normal(&p).unwrap(), nvec);
  // distance
  let r1 = Ray::new(&Vector3::new(0.0, 2.0, 0.0), &(-Vector3::EY));
  assert_eq!(p1.distance(&r1.unwrap()), Some((Vector2::O, 1.0, None)));
  //let p4 = Plane::new(&Vector3::EY, 0.0);
  let r1 = Ray::new(&Vector3::new(0.0, 2.0, 0.0), &Vector3::EX); // 光が平面に並行
  assert_eq!(p1.distance(&r1.unwrap()), None);
  let r1 = Ray::new(&Vector3::new(0.0, 5.0, 0.0), &nvec); // 光の後ろにある
  assert_eq!(p1.distance(&r1.unwrap()), None);
  // n_surface
  assert_eq!(p1.n_surface(), 1);
  assert_eq!(p2.n_surface(), 1);
  // surface_area
  assert_eq!(p1.surface_area(), f64::INFINITY);
  assert_eq!(p2.surface_area(), f64::INFINITY);
  // random_point
  assert_eq!(p1.random_point(), Ray::new(&Vector3::O, &nvec).unwrap());
}

#[test]
fn test_sphere() {
  // new
  let s1 = Shape::new_sphere(&Vector3::new(2.0, 0.0, 0.0), 2.0);
  if let Shape::Sphere {center, radius, radius2} = s1 {
    assert_eq!(center, Vector3::new(2.0, 0.0, 0.0));
    assert_eq!(radius, 2.0);
    assert_eq!(radius2, 4.0);
  };
  // get_normal
  assert_eq!(s1.get_normal(&Vector3::new(0.0, 0.0, 0.0)).unwrap(), -Vector3::EX);
  let rt2 = f64::sqrt(2.0);
  let v1 = s1.get_normal(&Vector3::new(2.0+2.0*rt2, 2.0*rt2, 2.0*rt2)).unwrap();
  let v2 = Vector3::new(1.0, 1.0, 1.0).normalize().unwrap();
  assert!(v1.near(&v2));
  // distance
  let r1 = Ray::new(&Vector3::new(2.0, 0.0, -3.5), &Vector3::EZ).unwrap(); // Z軸に真っ直ぐ
  assert_eq!(s1.distance(&r1), Some((Vector2::O, 1.5, None))); // 球より1.5後方
  let r1 = Ray::new(&Vector3::new(2.0, 0.0, -4.0), &Vector3::new(-2.0, 0.0, 4.0).normalize().unwrap()).unwrap(); // 外から原点へ
  assert_eq!(s1.distance(&r1), Some((Vector2::O, 2.6832815729997463, None)));
  let r1 = Ray::new(&Vector3::new(2.0, 0.0, 0.0), &Vector3::EZ).unwrap(); // 内部から出る光
  assert_eq!(s1.distance(&r1), Some((Vector2::O, 2.0, None))); // 半径と同じはず
  let r1 = Ray::new(&Vector3::new(2.0, 0.0, -4.0), &(-Vector3::EZ)).unwrap(); // 光の後ろにある
  assert_eq!(s1.distance(&r1), None); // 交点なし
  let r1 = Ray::new(&Vector3::new(2.0, 0.0, -4.0), &Vector3::EY).unwrap(); // 光と交差しない
  assert_eq!(s1.distance(&r1), None); // 交点なし
  // n_surface
  assert_eq!(s1.n_surface(), 1);
  // surface_area
  assert_eq!(s1.surface_area(), PI4*2.0*2.0);
  // random_point
  for _i in 0..1000 {
    let p = s1.random_point();
    if let Shape::Sphere {center, ..} = s1 {
      assert!((f64::abs((p.pos - center).norm() - 2.0)) < NEARLY0);
    };
    let d = s1.get_normal(&p.pos).unwrap();
    assert!(p.dir.near(&d));
  }
}

#[test]
fn test_triangle() {
  // new
  let t1 = Shape::new_triangle(&Vector3::new(0.0, 0.0, 0.0), &Vector3::EY, &Vector3::new(1.0, 0.0, 1.0), &Vector3::new(2.0, 0.0, 0.0));
  assert_eq!(t1, Shape::Triangle {pos: Vector3::new(0.0, 0.0, 0.0), nvec: Vector3::EY, dir1: Vector3::new(1.0, 0.0, 1.0), dir2: Vector3::new(2.0, 0.0, 0.0)});
  let t2 = Shape::new_triangle_with_points(&Vector3::new(0.0, 0.0, 0.0), &Vector3::new(1.0, 0.0, 1.0), &Vector3::new(2.0, 0.0, 0.0));
  assert_eq!(t1, t2);
  let t3 = Shape::new_triangle_with_normal(&Vector3::new(0.0, 0.0, 0.0), &Vector3::new(1.0, 0.0, 1.0), &Vector3::new(2.0, 0.0, 0.0), &Vector3::new(0.0, -1.0, 0.0));
  assert_eq!(t3, Shape::Triangle {pos: Vector3::new(0.0, 0.0, 0.0), nvec: -Vector3::EY, dir1: Vector3::new(1.0, 0.0, 1.0), dir2: Vector3::new(2.0, 0.0, 0.0)});
  let t4 = Shape::new_triangle_with_points(&Vector3::new(0.0, 0.0, 0.0), &Vector3::new(1.0, 1.0, 1.0), &Vector3::new(1.0, 1.0, 0.0));
  assert_eq!(t4, Shape::Triangle {pos: Vector3::new(0.0, 0.0, 0.0), nvec: Vector3::new(-1.0, 1.0, 0.0).normalize().unwrap(), dir1: Vector3::new(1.0, 1.0, 1.0), dir2: Vector3::new(1.0, 1.0, 0.0)});
  // get_normal
  assert_eq!(t1.get_normal(&Vector3::new(0.5, 0.0, 0.5)).unwrap(), Vector3::new(0.0, 1.0, 0.0));
  assert_eq!(t1.get_normal(&Vector3::new(0.0, 0.0, 0.0)).unwrap(), Vector3::new(0.0, 1.0, 0.0));
  assert_eq!(t1.get_normal(&Vector3::new(1.0, 0.0, 0.0)).unwrap(), Vector3::new(0.0, 1.0, 0.0));
  assert_eq!(t4.get_normal(&Vector3::new(0.5, 0.5, 0.5)).unwrap(), Vector3::new(-1.0, 1.0, 0.0).normalize().unwrap());
  // distance
  let r1 = Ray::new(&Vector3::new(0.5, 2.0, 0.5), &(-Vector3::EY)).unwrap();
  assert_eq!(t1.distance(&r1), Some((Vector2::new(0.5, 0.0), 2.0, None)));
  let r1 = Ray::new(&Vector3::new(0.5, 2.0, 0.0), &(-Vector3::EY)).unwrap();
  assert_eq!(t1.distance(&r1), Some((Vector2::new(0.0, 0.25), 2.0, None)));
  let r1 = Ray::new(&Vector3::new(1.0, 2.0, 0.5), &(-Vector3::EY)).unwrap();
  assert_eq!(t1.distance(&r1), Some((Vector2::new(0.5, 0.25), 2.0, None)));
  let r1 = Ray::new(&Vector3::new(0.0, 2.0, 1.0), &(-Vector3::EY)).unwrap(); // ポリゴンと交差しない
  assert_eq!(t1.distance(&r1), None);
  let r1 = Ray::new(&Vector3::new(0.0, 2.0, 0.0), &(-Vector3::EZ)).unwrap(); // ポリゴンに並行
  assert_eq!(t1.distance(&r1), None);
  // n_surface
  assert_eq!(t1.n_surface(), 1);
  // surface_area
  assert_eq!(t1.surface_area(), 2.0 * 1.0 / 2.0);
  assert_eq!(t4.surface_area(), f64::sqrt(1.0 + 1.0) * 1.0 / 2.0);
  // random_point
  for _i in 0..1000 {
    let p = t1.random_point();
    assert_eq!(p.dir, Vector3::EY);
    assert_eq!(p.pos.y, 0.0);
    // x1, y1 = (1.0, 1.0)
    // x2, y2 = (2.0, 0.0)
    let a = (2.0 * p.pos.z) / (2.0 * 1.0 - 1.0 * 0.0);
    let b = (p.pos.x - a * 1.0) / 2.0;
    assert!(a >= 0.0 && a <= 1.0 && b >= 0.0 && b <= 1.0 && a + b <= 1.0);
  }
}

#[test]
fn test_parallelogram() {
  // new
  let t1 = Shape::new_parallelogram(&Vector3::new(0.0, 0.0, 0.0), &Vector3::EY, &Vector3::new(1.0, 0.0, 1.0), &Vector3::new(2.0, 0.0, 0.0));
  assert_eq!(t1, Shape::Parallelogram {pos: Vector3::new(0.0, 0.0, 0.0), nvec: Vector3::EY, dir1: Vector3::new(1.0, 0.0, 1.0), dir2: Vector3::new(2.0, 0.0, 0.0)});
  let t2 = Shape::new_parallelogram_with_points(&Vector3::new(0.0, 0.0, 0.0), &Vector3::new(1.0, 0.0, 1.0), &Vector3::new(2.0, 0.0, 0.0));
  assert_eq!(t1, t2);
  let t3 = Shape::new_parallelogram_with_normal(&Vector3::new(0.0, 0.0, 0.0), &Vector3::new(1.0, 0.0, 1.0), &Vector3::new(2.0, 0.0, 0.0), &Vector3::new(0.0, -1.0, 0.0));
  assert_eq!(t3, Shape::Parallelogram {pos: Vector3::new(0.0, 0.0, 0.0), nvec: -Vector3::EY, dir1: Vector3::new(1.0, 0.0, 1.0), dir2: Vector3::new(2.0, 0.0, 0.0)});
  let t4 = Shape::new_parallelogram_with_points(&Vector3::new(0.0, 0.0, 0.0), &Vector3::new(1.0, 1.0, 1.0), &Vector3::new(1.0, 1.0, 0.0));
  assert_eq!(t4, Shape::Parallelogram {pos: Vector3::O, nvec: Vector3::new(-1.0, 1.0, 0.0).normalize().unwrap(), dir1: Vector3::new(1.0, 1.0, 1.0), dir2: Vector3::new(1.0, 1.0, 0.0)});
  // get_normal
  assert_eq!(t1.get_normal(&Vector3::new(0.5, 0.0, 0.5)).unwrap(), Vector3::new(0.0, 1.0, 0.0));
  assert_eq!(t1.get_normal(&Vector3::new(0.0, 0.0, 0.0)).unwrap(), Vector3::new(0.0, 1.0, 0.0));
  assert_eq!(t1.get_normal(&Vector3::new(1.0, 0.0, 0.0)).unwrap(), Vector3::new(0.0, 1.0, 0.0));
  assert_eq!(t4.get_normal(&Vector3::new(0.5, 0.5, 0.5)).unwrap(), Vector3::new(-1.0, 1.0, 0.0).normalize().unwrap());
  // distance
  let r1 = Ray::new(&Vector3::new(0.5, 2.0, 0.5), &(-Vector3::EY)).unwrap();
  assert_eq!(t1.distance(&r1), Some((Vector2::new(0.5, 0.0), 2.0, None)));
  let r1 = Ray::new(&Vector3::new(2.0, 2.0, 0.5), &(-Vector3::EY)).unwrap();
  assert_eq!(t1.distance(&r1), Some((Vector2::new(0.5, 0.75), 2.0, None)));
  let r1 = Ray::new(&Vector3::new(0.5, 2.0, 0.0), &(-Vector3::EY)).unwrap();
  assert_eq!(t1.distance(&r1), Some((Vector2::new(0.0, 0.25), 2.0, None)));
  let r1 = Ray::new(&Vector3::new(1.0, 2.0, 0.5), &(-Vector3::EY)).unwrap();
  assert_eq!(t1.distance(&r1), Some((Vector2::new(0.5, 0.25), 2.0, None)));
  let r1 = Ray::new(&Vector3::new(0.0, 2.0, 1.0), &(-Vector3::EY)).unwrap(); // ポリゴンと交差しない
  assert_eq!(t1.distance(&r1), None);
  let r1 = Ray::new(&Vector3::new(0.0, 2.0, 0.0), &(-Vector3::EZ)).unwrap(); // ポリゴンに並行
  assert_eq!(t1.distance(&r1), None);
  // n_surface
  assert_eq!(t1.n_surface(), 1);
  // surface_area
  assert_eq!(t1.surface_area(), 2.0 * 1.0);
  assert_eq!(t4.surface_area(), f64::sqrt(1.0 + 1.0) * 1.0);
  // random_point
  for _i in 0..1000 {
    let p = t1.random_point();
    assert_eq!(p.dir, Vector3::EY);
    assert_eq!(p.pos.y, 0.0);
    // x1, y1 = (1.0, 1.0)
    // x2, y2 = (2.0, 0.0)
    let a = (2.0 * p.pos.z) / (2.0 * 1.0 - 1.0 * 0.0);
    let b = (p.pos.x - a * 1.0) / 2.0;
    assert!(a >= 0.0 && a <= 1.0 && b >= 0.0 && b <= 1.0 && a + b <= 2.0);
  }
}

// Mesh (三角ポリゴンメッシュ)  --------------------------------------------------

// 頂点型： vtx=頂点座標番号, nvec=法線ベクトル番号, map=マッピング座標番号
#[derive(Debug, PartialEq, Clone)]
struct Vertex(usize, usize, usize);

#[derive(Debug, PartialEq, Clone)]
pub struct Patch(Vertex, Vertex, Vertex);

impl Patch {
  pub const DUMMY: Patch = Patch(Vertex(0, 0, 0), Vertex(0, 0, 0), Vertex(0, 0, 0));
}

#[derive(Debug, PartialEq, Clone)]
pub struct Mesh {
  patches : Vec<Patch>,
  vertexes: Vec<Position3>,
  normals : Vec<Direction3>,
  uvmaps  : Vec<Vector2>,
}

impl Mesh {

}

/*
impl Shape for Mesh {
  fn get_normal(&self, _pos: &Position3)  -> Option<Direction3> {
    None
  }

  fn distance(&self, ray: &Ray) -> Option<(Vector2, Flt, Option<usize>)> {
    fn comp_triangle() -> Option<(Vector2, Flt, Patch, Direction3, Direction3)> {

    }

    fn build_triangle() -> Triangle {

    } 

    let mut t = f64::NEG_INFINITY;
    let mut idx: usize = 0;
    let mut uv = Vector2::O;
    for pat in self.patches.iter() {

    };

    if t < NEARLY0 {
      None
    } else {
      Some((uv, t, Some(idx)))
    }
  }

  fn n_surface(&self) -> usize {
    self.patches.len()
  }

  fn surface_area(&self) -> Flt {
    fn patch_area(mesh: &Mesh, pat: &Patch) -> Flt {
      let d1 = mesh.vertexes[pat.1.0] - mesh.vertexes[pat.0.0];
      let d2 = mesh.vertexes[pat.2.0] - mesh.vertexes[pat.0.0];
      d1.cross(&d2).norm() / 2.0
    }

    self.patches
      .iter()
      .fold(0.0, |area, pat| area + patch_area(self, &pat))
  }

  fn random_point(&self) -> Ray {

  }

}
*/

#[test]
fn test_mesh() {
  // new
  assert!(false);

}

// UTILS

// method_moller: Moller法による三角/四角ポリゴンの交差判定
//   judgement of intersection for polygon 
//   https://shikousakugo.wordpress.com/2012/07/01/ray-intersection-3/

pub fn method_moller(l: Flt, pos0: &Position3, dir1: &Direction3, dir2: &Direction3, ray: &Ray) -> Option<(Vector2, Flt)> {
  let re2 = ray.dir.cross(dir2);
  let det_a = re2.dot(dir1);
  if det_a == 0.0 {
    None
  } else {
    let pp = ray.pos - *pos0;
    let te1 = pp.cross(dir1);
    let u = re2.dot(&pp)  / det_a;
    let v = te1.dot(&ray.dir)  / det_a;
    let t = te1.dot(dir2) / det_a;
    if 0.0 <= u && u <= 1.0 &&
       0.0 <= v && v <= 1.0 &&
       u + v <= l {
      Some((Vector2::new(u, v), t))
    } else {
      None
    }
  }
}

// get_direction: 
pub fn get_direction(shp: &Shape, dstpos: &Position3, srcpos: &Position3) -> Option<Direction3> {
  match shp.get_normal(dstpos) {
    None => None,
    Some(nvec) => {
      let dir = *dstpos - *srcpos;
      let cos = nvec.dot(&dir);
      if cos > 0.0 {
        None
      } else {
        Some(dir)
      }
    }
  }
}

// utility functions
/*
fn distance_plane(r: &Ray, n: &Direction3, d: &Flt) -> Vec<Flt> {
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

fn distance_polygon(l: Flt, r: &Ray, p: &Position3, d1: &Direction3, d2: &Direction3) -> Vec<Flt> {
  let res = method_moller(l, p, d1, d2, &r.pos, &r.dir);
  if res == None {
    vec![]
  } else {
    vec![res.unwrap().2]
  }
}
*/



