// Photon tracer

use std::env;
//use std::io::{Error, ErrorKind};

use ppmpa::ray::*;
//use ppmpa::ray::algebra::*;
//use ppmpa::ray::geometry::*;
use ppmpa::ray::light::*;
use ppmpa::ray::object::*;
use ppmpa::ray::physics::*;
//use ppmpa::ray::optics::*;
use ppmpa::scene::*;
//use ppmpa::screen::*;
use ppmpa::tracer::*;

const USAGE: &str = "Usage: pm [-c|-h] <scene file> [<#photon>] (output photon map to stdout)";
const DEF_NPHOTON: i32 = 100_000;
const DEF_USECLASSIC: bool = true;

//fn main() -> Result<(), std::io::Error> {
fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() < 2 || args[1] == "-h" {
    println!("{}", USAGE);
    //return Err(std::io::Error::new(ErrorKind::Other, USAGE));
    return;
  }
  //let scr = read_screen(&args[1]);
  let nphoton = if args.len() == 3 {
    let np = args[2].parse::<i32>();
    match np {
      Ok(n) => n,
      _     => DEF_NPHOTON,
    }
  } else {
    DEF_NPHOTON
  };
  let uc = DEF_USECLASSIC;
  let (lgts, objs) = read_scene(&args[1]);
  let power0: Flt = lgts.iter().fold(0.0, |power0, l| power0 + l.flux());
  let power = power0 / nphoton as Flt;
  let ns = lgts.iter().map(|l| calc_n(&power, l));   // 1光源あたりのフォトン数のリスト
  
  println!("{}", nphoton);
  println!("{}", power);

  for (n, l) in ns.zip(lgts.iter()) {
    output_photon_caches(&uc, &objs, &l, n)
  }
}

fn calc_n(power: &Flt, lgt: &Light) -> i64 {
  (lgt.flux() / power).round() as i64
}

fn output_photon_caches(uc: &bool, objs: &Vec<Object>, lgt: &Light, np: i64) {
  for _i in 0..np {
    output_photon_cache(uc, objs, lgt);
  }
}

fn output_photon_cache(uc: &bool, objs: &Vec<Object>, lgt: &Light) {
  let ph = lgt.generate_photon();
  let pcs = trace_photon(uc, &M_AIR, objs, 0, &ph);
  for pc in pcs {
    let w = match pc.wl {
      Wavelength::Red   => "Red",
      Wavelength::Green => "Green",
      Wavelength::Blue  => "Blue",
    };
    println!("{} {} {} {} {} {} {}", w,
      pc.ray.pos.v[0], pc.ray.pos.v[1], pc.ray.pos.v[2],
      pc.ray.dir.v[0], pc.ray.dir.v[1], pc.ray.dir.v[2]);
  }
}

