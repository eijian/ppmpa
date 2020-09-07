// Photon tracer

use std::env;
//use std::io::{Error, ErrorKind};

use ppmpa::ray::*;
use ppmpa::ray::algebra::*;

const USAGE: &str = "Usage: pm <screen file> <scene file>  (output photon map to stdout)";

//fn main() -> Result<(), std::io::Error> {
fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() != 2 {
    println!("{}", USAGE);
    //return Err(std::io::Error::new(ErrorKind::Other, USAGE));
    return;
  }
  let v1 = Vector3::new(1.0, 2.0, 3.0);

  println!("NRL={}", NEARLY0);
  println!("V1={}", v1);

  //Ok(())
}

