// test of Kd-Tree


use kdtree::KdTree;
use kdtree::ErrorKind;
use kdtree::distance::squared_euclidean;

use ppmpa::ray::algebra::*;
use ppmpa::ray::geometry::*;
use ppmpa::ray::optics::*;
use ppmpa::ray::physics::*;



fn main() {
  let dimensions = 3;
  let mut kdtree = KdTree::new(dimensions);

  let p1 = Photon::new(&Wavelength::Red, &Ray::new(&Vector3::new(0.0, 0.0, 0.0), &Vector3::EY));
  let p2 = Photon::new(&Wavelength::Green, &Ray::new(&Vector3::new(1.0, 1.0, 0.0), &Vector3::EX));
  let p3 = Photon::new(&Wavelength::Blue, &Ray::new(&Vector3::new(1.0, 1.0, 1.0), &Vector3::EZ));
  let p4 = Photon::new(&Wavelength::Red, &Ray::new(&Vector3::new(-1.0, -1.0, -1.0), &Vector3::EY));
  let p5 = Photon::new(&Wavelength::Green, &Ray::new(&Vector3::new(2.0, 2.0, 2.0), &Vector3::EX));
  let p6 = Photon::new(&Wavelength::Blue, &Ray::new(&Vector3::new(2.0, 2.0, 2.0), &Vector3::EZ));

  kdtree.add(&p1.ray.pos.v, &p1).unwrap();
  kdtree.add(&p2.ray.pos.v, &p2).unwrap();
  kdtree.add(&p3.ray.pos.v, &p3).unwrap();
  kdtree.add(&p4.ray.pos.v, &p4).unwrap();
  kdtree.add(&p5.ray.pos.v, &p5).unwrap();
  kdtree.add(&p6.ray.pos.v, &p6).unwrap();

  println!("SIZE:{}", kdtree.size());
  println!("\nNEAREST 2 Photons");
  for p in kdtree.nearest(&p1.ray.pos.v, 2, &squared_euclidean).unwrap().iter() {
    println!("P: {:?}", p);
  }

  println!("\nNEAREST 3 Photons");
  for p in kdtree.nearest(&p1.ray.pos.v, 3, &squared_euclidean).unwrap().iter() {
    println!("P: {:?}", p);
  }

  println!("\nWITHIN 2.5");
  for p in kdtree.within(&p1.ray.pos.v, 2.5, &squared_euclidean).unwrap().iter() {
    println!("P: {:?}", p);
  }

  println!("\nWITHIN 12.0");
  for p in kdtree.within(&p1.ray.pos.v, 3.5*3.5, &squared_euclidean).unwrap().iter() {
    println!("P: {:?}", p);
  }





}

