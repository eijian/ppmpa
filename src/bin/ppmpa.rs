// Progressive Photon Mapping with Probability Approach
// 

use std::env;

use ppmpa::camera::*;
use ppmpa::photonmap::*;
use ppmpa::ray::*;
use ppmpa::ray::light::*;
use ppmpa::ray::object::*;
use ppmpa::ray::optics::*;
use ppmpa::scene::*;
use ppmpa::tracer::*;

const USAGE: &str = "Usage: ppmpa [-nc|-h] <#photon> <radius> <camera file> <scene file>";
const DEF_USECLASSIC: bool = true;
const DEF_NPHOTON: i32 = 100000;
const DEF_RADIUS: Flt = 0.1;

//fn main() -> Result<(), std::io::Error> {
fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() < 5 || args[1] == "-h" {
    print_usage();
    return;
    //return Err(std::io::Error::new(ErrorKind::Other, USAGE));
  }

  let (nphoton, radius, uc, cam, lgts, objs) = get_params(&args);
  let power: Flt = lgts.iter().fold(0.0, |power0, l| power0 + l.flux()) / nphoton as Flt;
    let ns = lgts.iter().map(|l| calc_n(&power, l)).collect();   // 1光源あたりのフォトン数のリスト
  
  // iteration
  let image = iteration(&uc, &power, &ns, &radius, &cam, &objs, &lgts);

  for l in cam.pnm_header() {
    println!("{}", l);
  }
  for c in &image {
    println!("{}", radiance_to_string(c));
  }
}

fn print_usage() {
  eprintln!("{}", USAGE);
}

fn get_params(args: &Vec<String>) -> (i32, Flt, bool, Camera, Vec<Light>, Vec<Object>) {

  let (argoffset, uc) = if args[1] == "-nc" {  // No use classic
    (2, !DEF_USECLASSIC)
  } else {
    (1, DEF_USECLASSIC)
  };

  let nphoton = match args[argoffset].parse::<i32>() {
    Ok(np) => np,
    _      => DEF_NPHOTON,
  };
  let radius  = match args[argoffset + 1].parse::<Flt>() {
    Ok(r) => r * r,
    _     => DEF_RADIUS * DEF_RADIUS,
  };
  let cam = read_camera(&args[argoffset + 2]);
  let (lgts, objs) = read_scene(&args[argoffset + 3]);

  (nphoton, radius, uc, cam, lgts, objs)
}

fn calc_n(power: &Flt, lgt: &Light) -> i64 {
  (lgt.flux() / power).round() as i64
}

fn iteration(uc: &bool, pw: &Flt, ns: &Vec<i64>, radius: &Flt, cam: &Camera, objs: &Vec<Object>, lgts: &Vec<Light>) -> Vec<Radiance> {
  let mut phs: Vec<Photon> = vec![];
  for (n, l) in ns.iter().zip(lgts.iter()) {
    phs.extend(get_photon_caches(&uc, &objs, &l, *n));
  }
  let (_msize, pmap) = build_photonmap(pw, radius, &phs, &cam.n_sample_photon);

  let rays = cam.screen_map.iter().map(|p| cam.generate_ray(p));
  let imgs = rays.map(|r| trace_ray(cam, &M_AIR, 0, objs, lgts, &r, &pmap, radius, uc)).collect();
  imgs
}

fn get_photon_caches(uc: &bool, objs: &Vec<Object>, lgt: &Light, np: i64) -> Vec<Photon> {
  let mut phs: Vec<Photon> = vec![];
  for _i in 0..np {
    phs.extend(trace_photon(uc, &M_AIR, objs, 0, &lgt.generate_photon()));
  }
  phs
}


