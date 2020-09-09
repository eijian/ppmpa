// Photon tracer

use std::env;
//use std::io::{Error, ErrorKind};

use ppmpa::ray::*;
use ppmpa::ray::algebra::*;
use ppmpa::screen::*;

const USAGE: &str = "Usage: rtc <screen file> <scene file>";

//fn main() -> Result<(), std::io::Error> {
fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() != 2 {    
    println!("{}", USAGE);
    //return Err(std::io::Error::new(ErrorKind::Other, USAGE));
    return;
  }
  let scr = read_screen(&args[0]);
  //let (lgts, objs) = read_scene(args[1]);
  let v1 = Vector3::new(1.0, 2.0, 3.0);

  println!("NRL={}", NEARLY0);
  println!("V1={}", v1);
  println!("#photon={}", scr.nphoton);
  //Ok(())
}

