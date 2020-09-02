

use ppmpa::ray::algebra::Vector3;
use ppmpa::ray::algebra::NEARLY0;


fn main() {
  let v1 = Vector3(1.0, 2.0, 3.0);
  let v2 = Vector3(3.0, 2.0, 1.0);

  println!("Hello, world!");
  println!("nearly = {}", NEARLY0);
  println!("neg = {:?}", -v1);
  println!("v = {:?}", v1 - v2);
  println!("v*3.3= {:?}", v2 * 3.3);
  println!("2.2*v= {:?}", 2.2 * v1);
  let v3 = v1 / 2.0;
  println!("v / 2= {:?}", v3);
  let v4 = v2 / 0.0;
  println!("v / 0= {:?}", v4);
  println!("v1.v2= {}", v1.dot(v2));
  println!("|v1|= {}", v1.normalize());
}
