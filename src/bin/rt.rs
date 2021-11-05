// Photon tracer

use std::env;
use std::time::{Instant};

use ppmpa::ray::*;
//use ppmpa::ray::algebra::*;
//use ppmpa::ray::physics::*;
//use ppmpa::ray::geometry::*;
use ppmpa::camera::*;
use ppmpa::ray::optics::*;
use ppmpa::photonmap::*;
use ppmpa::scene::*;
use ppmpa::tracer::*;

const USAGE: &str = "Usage: rtc <scene file> <camera file> [<radius>]";
const DEF_USECLASSIC: bool = true;
const DEF_RADIUS: Flt = 0.1;

fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() < 3 {
    println!("{}", USAGE);
    //return Err(std::io::Error::new(ErrorKind::Other, USAGE));
    return;
  }
  let uc = DEF_USECLASSIC;  // use classic for direct
  let radius = if args.len() == 4 {
    let r = args[3].parse::<Flt>();
    match r {
      Ok(r) => r * r,
      _     => DEF_RADIUS * DEF_RADIUS,
    }
  } else {
    DEF_RADIUS * DEF_RADIUS
  };
  let (lgts, objs) = read_scene(&args[1]);
  let cam = read_camera(&args[2]);

  let t0 = Instant::now();
  let (msize, photonmap) = read_map(&cam.n_sample_photon, &radius);
  let t1 = t0.elapsed();
  eprintln!("finished reading map: {} photons, {:?}.", msize, t1);

  let rays = cam.screen_map.iter().map(|p| cam.generate_ray(p));
  let image: Vec<Radiance> = rays.map(|r| trace_ray(&cam, &M_AIR, 0, &objs, &lgts, &r, &photonmap, &radius, &uc)).collect();

  for l in cam.pnm_header() {
    println!("{}", l);
  }
  if cam.progressive == false {
    for c in &image {
      println!("{}", rgb_to_string(&cam.radiance_to_rgb(c)));
    }
  } else {
    for c in &image {
      println!("{}", radiance_to_string(c));
    }
  }



/*
  let v1 = Vector3::new(1.0, 2.0, 3.0);
  let v2 = Vector3::new(3.0, 2.0, 1.0);

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
  println!("d3.x={}", d3.v[0]);
*/

}
