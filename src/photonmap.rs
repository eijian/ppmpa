// photonmap

use std::io;
use std::io::BufReader;
use std::io::prelude::*;

use kdtree::KdTree;

use super::ray::*;
use super::ray::geometry::*;
//use super::ray::light::*;
//use super::ray::object::*;
use super::ray::optics::*;
use super::ray::physics::*;

pub struct PhotonMap {
  pub power: Flt,
  pub nsample: i32,
  pub radius: Flt,
  pub kdtree: KdTree<Flt, Photon, [Flt; 3]>,
}

pub fn build_photonmap(pw: &Flt, radius: &Flt, phs: &Vec<Photon>, nsample: &i32) -> (usize, PhotonMap) {
  let mut pmap = KdTree::new(3);
  for p in phs {
    pmap.add(p.ray.pos.v, *p).unwrap();
  }
  (pmap.size(), PhotonMap {power: *pw, nsample: *nsample, radius: *radius, kdtree: pmap})
}

pub fn read_map(nsample: &i32, radius: &Flt) -> (usize, PhotonMap) {
  //eprintln!("radius= {}", radius);
  let mut reader = BufReader::new(io::stdin());
  let mut line1 = String::new();
  reader.read_line(&mut line1).expect("invalid #photon");
  let mut line2 = String::new();
  let pw0 = match reader.read_line(&mut line2) {
    Ok(_) => {
      match line2.trim().parse::<Flt>() {
        Ok(pw) => pw,
        _ => 1.0,
      }
    },
    _ => 1.0,
  };
  let mut pmap =  KdTree::new(3);
  let mut contents = String::new();
  match reader.read_to_string(&mut contents) {
    Err(e) => panic!("Error in reading photon map: {:?}", e),
    _      => (),
  }
  let mut phs: Vec<Photon> = vec![];
  let mut elems: Vec<&str>;
  for line in contents.lines() {
    elems = line.split(' ').collect();
    let wl = match elems[0] {
      "Red"   => Wavelength::Red,
      "Green" => Wavelength::Green,
      "Blue"  => Wavelength::Blue,
      _       => Wavelength::Red,
    };
    let px = elems[1].parse::<Flt>().unwrap();
    let py = elems[2].parse::<Flt>().unwrap();
    let pz = elems[3].parse::<Flt>().unwrap();
    let dx = elems[4].parse::<Flt>().unwrap();
    let dy = elems[5].parse::<Flt>().unwrap();
    let dz = elems[6].parse::<Flt>().unwrap();
    phs.push(Photon::new(&wl, &Ray::new_from_elem(px, py, pz, dx, dy, dz).unwrap()));
  }
  for p in phs {
    pmap.add(p.ray.pos.v, p).unwrap();
  }
  (pmap.size(), PhotonMap {power: pw0, nsample: *nsample, radius: *radius, kdtree: pmap})
}

/*
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
*/


