// Photon tracer

use std::env;
//use std::io::{Error, ErrorKind};

//use ppmpa::ray::*;
//use ppmpa::ray::algebra::*;
use ppmpa::ray::geometry::*;
use ppmpa::ray::optics::*;
use ppmpa::camera::*;
use ppmpa::scene::*;
use ppmpa::tracer::*;

const USAGE: &str = "Usage: rtc <screen file> <scene file>";

//fn main() -> Result<(), std::io::Error> {
fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() != 3 {
    println!("{}", USAGE);
    //return Err(std::io::Error::new(ErrorKind::Other, USAGE));
    return;
  }
  let scr = read_camera(&args[1]);
  for l in scr.pnm_header() {
    println!("{}", l);
  }

  let rays: Vec<Ray> = scr.screen_map.iter().map(|p| scr.generate_ray(p)).collect();
  let (lgts, objs) = read_scene(&args[2]);
  let image: Vec<Radiance> = rays.iter().map(|r| trace_ray_classic(&scr, &M_AIR, 0, &objs, &lgts, &r)).collect();
  for c in &image {
    if scr.progressive == false {
      println!("{}", rgb_to_string(&scr.radiance_to_rgb(c)));
    } else {
      println!("{}", radiance_to_string(c));
    }
  }
  /*
  let v1 = Vector3::new(1.0, 2.0, 3.0);

  println!("NRL={}", NEARLY0);
  println!("V1={}", v1);
  println!("#photon={}", scr.nphoton);
  */
  //Ok(())
}

