

use ppmpa::ray::*;
use ppmpa::ray::algebra::*;
use ppmpa::ray::physics::*;

fn main() {
  let v1 = Vector3(1.0, 2.0, 3.0);
  let v2 = Vector3(3.0, 2.0, 1.0);

  println!("Hello, world!");
  println!("nearly = {}", NEARLY0);
  let v3 = v1 / 2.0;
  println!("v / 2= {:?}", v3);
  let v4 = v2 / 0.0;
  println!("v / 0= {:?}", v4);
  println!("v1.v2= {}", v1.dot(&v2));
  let v5 = v1.normalize();
  println!("v1/|v1|= {:?}", v5);
  let l1 = v1.norm();
  println!("v1 length={:?}", l1);
  if let Some(v6) = v5 {
    let l6 = v6.norm();
    println!("v6 length={:?}", l6);
  }

  println!("EX={:?}", Vector3::EX);
  println!("EX x EY={:?}", Vector3::EX.cross(&Vector3::EY));

  let p1 = Position3::new_pos(1.1, 2.1, 3.1);
  println!("p1={:?}", p1);
  let d1 = Direction3::new_dir(1.2, 2.2, 3.2);
  println!("d1={:?}", d1);
  let d2 = Direction3::new_dir_from_angle(1.5, 2.3);
  println!("d2={:?}", d2);
  let d3 = generate_random_dir();
  println!("d3={:?}", d3);
  println!("d3.x={}", d3.0);


}
