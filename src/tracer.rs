// tracer

use std::f64;
use std::iter::*;
use std::vec::*;
use std::cmp::*;

//use kdtree::KdTree;
use kdtree::distance::squared_euclidean;

use super::ray::*;
use super::ray::algebra::*;
use super::ray::geometry::*;
use super::ray::object::*;
use super::ray::optics::*;
use super::ray::light::*;
use super::ray::material::*;
use super::ray::physics::*;
use super::ray::surface::*;
//use super::ray::optics::*;

use super::camera::*;
use super::scene::*;

const ONE_PI: Flt  = 1.0 / f64::consts::PI;
const SR_HALF: Flt = 1.0 / (2.0 * f64::consts::PI);

const MAX_TRACE: i32 = 10;

// Photon tracing

pub fn trace_photon(uc: &bool, m0: &Material, objs: &Vec<Object>, l: i32, ph: &Photon) -> Vec<PhotonCache> {
  if l >= MAX_TRACE {
    return vec![]
  }
  
  let is = calc_intersection(&ph.ray, objs);
  if is == None {
    return vec![]
  }
  let is1 = is.unwrap();
  let sf = is1.mate.surface;
  let mut pcs = match sf {
    Surface::Simple {
      reflectance,
      specular_refl,
      diffuseness,
      metalness,
      roughness,
      density_pow,
    } => {
      let rn = sf.roughness();
      let i = russian_roulette(&[rn]);
      match i {
        0 => reflect_diff(uc, m0, objs, l, ph, &is1),
        _ => reflect_spec(uc, m0, objs, l, ph, &is1),
      }
    },
    Surface::TS {
      albedo_diff,
      albedo_spec,
      scatterness,
      metalness,
      roughness,
      density_pow,
      alpha,
    } => {
      let eta = relative_ior_wavelength(&m0.ior, &is1.mate.ior, &ph.wl);
      match sf.next_direction(&eta, &is1.nvec, &ph.ray.dir, &ph.wl) {
        Some((dir, m)) => {
          let mate = if m == true { m0 } else { &is1.mate };
          trace_photon(uc, mate, objs, l+1, &Photon::new(&ph.wl, &Ray::new(&is1.pos, &dir)))
        },
        None      => vec![],
      }
      
    },
    _ => vec![],
  };
  if (*uc == false || l > 0) && sf.store_photon() == true {
    pcs.push(Photon::new(&ph.wl, &Ray::new(&is1.pos, &ph.ray.dir)));
  }
  pcs
}

fn reflect_diff(uc: &bool, m0: &Material, objs: &Vec<Object>, l: i32, ph: &Photon, is: &Intersection) -> Vec<PhotonCache> {
  let i = russian_roulette(&[is.mate.surface.albedo_diff(&ph.wl)]);
  match i {
    0 => {
      let dr = diffuse_reflection(&is.nvec);
      trace_photon(uc, m0, objs, l+1, &Photon::new(&ph.wl, &Ray::new(&is.pos, &dr)))
    },
    _ => vec![],
  }
}

fn reflect_spec(uc: &bool, m0: &Material, objs: &Vec<Object>, l: i32, ph: &Photon, is: &Intersection) -> Vec<PhotonCache> {
  let (rdir, cos1) = specular_reflection(&is.nvec, &ph.ray.dir);

  let f = schlick(&is.mate.surface.albedo_spec(&ph.wl), &cos1);
  let j = russian_roulette(&[f]);
  match j {
    0 => trace_photon(uc, m0, objs, l+1, &Photon::new(&ph.wl, &Ray::new(&is.pos, &rdir))),
    _ => {
      if is.mate.ior.wavelength(&ph.wl) == 0.0 {
        vec![]
      } else {
        reflect_trans(uc, m0, objs, l, ph, is, &cos1)
      }
    },
  }
}

fn reflect_trans(uc: &bool, m0: &Material, objs: &Vec<Object>, l: i32, ph: &Photon, is: &Intersection, c0: &Flt) -> Vec<PhotonCache> {
  let eta = relative_ior_wavelength(&m0.ior, &is.mate.ior, &ph.wl);
  let (tdir, cos2) = specular_refraction(&is.nvec, &ph.ray.dir, &eta);
  match tdir {
    Some(tdir) => {
      let m02 = if tdir.dot(&is.nvec) < 0.0 {
        is.mate
      } else {
        M_AIR
      };
      trace_photon(uc, &m02, objs, l+1, &Photon::new(&ph.wl, &Ray::new(&is.pos, &tdir)))
    },
    None => vec![]
  }
}

// Photon mapping method

pub fn trace_ray(cam: &Camera, m0: &Material, l: i32, objs: &Vec<Object>, lgts: &Vec<Light>, r: &Ray, pmap: &PhotonMap, radius: &Flt, uc: &bool) -> Radiance {
  if l >= MAX_TRACE { return Radiance::RADIANCE0 }
  let is = calc_intersection(r, objs);
  if is == None { return Radiance::RADIANCE0 }
  let is1 = is.unwrap();
  
  // L_diffuse
  let mut di = if *uc {
    let mut rad = Radiance::RADIANCE0;
    for lt in lgts {
      rad = rad + get_radiance_from_light(objs, &is1.pos, &is1.nvec, lt);
    }
    rad
  } else {
    Radiance::RADIANCE0
  };
  di = di + estimate_radiance(&radius, &cam, &pmap, &is1);
   
  let mate = is1.mate;

  // L_spec
  let (rdir0, cos1) = specular_reflection(&is1.nvec, &r.dir);
  let rdir = reflection_glossy(&is1.nvec, &rdir0, &mate.surface.power_glossy());
  let si = if mate.surface.reflect(&cos1) == true {
    trace_ray(cam, m0, l+1, objs, lgts, &Ray::new(&is1.pos, &rdir), pmap, radius, uc)
  } else {
    Radiance::RADIANCE0
  };
  
  // L_trans
  let eta = relative_ior_average(&m0.ior, &mate.ior);
  let hvec = (rdir - r.dir).normalize().unwrap();
  let (tdir, cos2) = specular_refraction(&hvec, &r.dir, &eta);
  let ti = match tdir {
    Some(tdir) if mate.surface.refract(&cos1) == true => {
      let m02 = match is1.io {
        InOut::In  => mate,
        InOut::Out => M_AIR,
      };
      trace_ray(cam, &m02, l+1, objs, lgts, &Ray::new(&is1.pos, &tdir), pmap, radius, uc)
    },
    _ => Radiance::RADIANCE0,
  };

  let cos = if cos1 < cos2 { cos1 } else { cos2 };

  mate.emittance * SR_HALF +
  mate.surface.bsdf(&is1.nvec, &r.dir, &rdir, &tdir, &cos, &eta, &di, &si, &ti)
}

fn estimate_radiance(radius: &Flt, cam: &Camera, pmap: &PhotonMap, is: &Intersection) -> Radiance {
  let ps: Vec<(Flt, &Photon)> = pmap.kdtree.within(&is.pos.v, pmap.radius, &squared_euclidean).unwrap();
  if ps.len() == 0 {
    Radiance::RADIANCE0
  } else {
    let mut rad = Radiance::RADIANCE0;
    for (d, p2) in ps {
      let wt = match cam.pfilter {
        PhotonFilter::Non   => 1.0,
        PhotonFilter::Cone  => filter_cone(&d, &radius),
        PhotonFilter::Gauss => filter_gauss(&d, &radius),
      };
      rad = rad + photoninfo_to_radiance(&is.nvec, &(wt * pmap.power), p2);
    }
    rad * (ONE_PI / radius)
  }
}

// Cone filter
const K_CONE: Flt = 1.1;
const FAC_K: Flt  = 1.0 - 2.0 / (3.0 * K_CONE);

fn filter_cone(d: &Flt, rmax: &Flt) -> Flt {
  let d2 = f64::sqrt(*d / rmax) / K_CONE;
  if d2 > 1.0 { 0.0 } else { (1.0 - d2) / FAC_K }
}

// Gauss filter
//const ALPHA: Flt  = 1.0 / 0.918;
const ALPHA: Flt  = 0.918;
const BETA: Flt   = 1.953;
const E_BETA: Flt = 1.0 - 0.14184788965323;  // 1 - exp(-β)
const CORR: Flt   = 0.5; //0.355;

fn filter_gauss(d: &Flt, rmax: &Flt) -> Flt {
  let e_r = 1.0 - (-BETA * d / (rmax * 2.0)).exp();
  if e_r > E_BETA { 0.0 } else { ALPHA * (1.0 - e_r / E_BETA) + CORR }
}

//
// CLASSIC Ray tracer
//
pub fn trace_ray_classic(cam: &Camera, m0: &Material, l: i32, objs: &Vec<Object>, lgts: &Vec<Light>, r: &Ray) -> Radiance {
  if l >= 10 {
    return Radiance::RADIANCE0
  }
  let is = calc_intersection(r, objs);
  if is == None {
    return Radiance::RADIANCE0
  }

  let is1 = is.unwrap();
  let mate = is1.mate;
  let (rdir, cos1) = specular_reflection(&is1.nvec, &r.dir);

  let mut di = Radiance::RADIANCE0;
  for lt in lgts {
    di = di + get_radiance_from_light(objs, &is1.pos, &is1.nvec, lt);
  }
  di = di + cam.ambient; 

  let si = if mate.surface.reflect(&cos1) == true {
    trace_ray_classic(cam, m0, l+1, objs, lgts, &Ray::new(&is1.pos, &rdir))
  } else {
    Radiance::RADIANCE0
  };

  let eta = relative_ior_average(&m0.ior, &mate.ior);
  let (tdir, cos2) = specular_refraction(&is1.nvec, &r.dir, &eta);
  let ti = match tdir {
    Some(tdir) if mate.surface.refract(&cos1) == true => {
      let m02 = if tdir.dot(&is1.nvec) < 0.0 { mate } else { M_AIR };
      trace_ray_classic(cam, &m02, l+1, objs, lgts, &Ray::new(&is1.pos, &tdir))
    },
    _ => Radiance::RADIANCE0,
  };

  // L_trans
  let eta = relative_ior_average(&m0.ior, &mate.ior);
  let (tdir, cos2) = specular_refraction(&is1.nvec, &r.dir, &eta);
  let ti = match tdir {
    Some(tdir) if mate.surface.refract(&cos1) == true => {
      let m02 = if tdir.dot(&is1.nvec) < 0.0 { mate } else { M_AIR };
      trace_ray_classic(cam, &m02, l+1, objs, lgts, &Ray::new(&is1.pos, &tdir))
    },
    _ => Radiance::RADIANCE0,
  };
  
  mate.emittance * SR_HALF +
  mate.surface.bsdf(&is1.nvec, &r.dir, &rdir, &tdir, &cos1, &eta, &di, &si, &ti)
}

// private

fn get_radiance_from_light(objs: &Vec<Object>, p: &Position3, n: &Direction3, l: &Light) -> Radiance {
  let (dists, coss): (Vec<Flt>, Vec<Flt>) = illuminated(objs, p, n, &l.get_direction(p)).iter().cloned().unzip();
  let mut rad = Radiance::RADIANCE0;
  for r in l.get_radiance(&dists).iter().zip(coss).map(|(a,b)| *a * b) {
    rad = rad + r;
  }
  rad
}

fn illuminated(os: &Vec<Object>, p: &Position3, n: &Direction3, lds: &Vec<Direction3>) -> Vec<(Flt, Flt)> {
  let mut ret: Vec<(Flt, Flt)> = vec![];
  for ld in lds {
    let ld2 = ld.normalize();
    if ld2 == None { continue; }
    let ld3 = ld2.unwrap();
    let cos0 = n.dot(&ld3);
    if cos0 < 0.0 { continue; }
    let lray = Ray::new(&p, &ld3);
    let is = calc_intersection(&lray, os);
    if is == None { continue; }
    let p2 = is.unwrap().pos;
    let sq_ldist = ld.square();
    let sq_odist = (p2 - *p).square();
    if sq_ldist - sq_odist > 0.002 { continue; }
    ret.push((sq_ldist, cos0 * cos0));
  }
  ret
}

#[derive(PartialEq)]
enum InOut {
  In,
  Out,
}

#[derive(PartialEq)]
struct Intersection {
  pub pos:  Position3,
  pub nvec: Direction3,
  pub mate: Material,
  pub io:   InOut,
}

fn calc_intersection(r: &Ray, os: &Vec<Object>) -> Option<Intersection> {
  fn sorting(is1: &(Flt, Object), is2: &(Flt, Object)) -> Ordering {
    if is1.0 < is2.0 {
      Ordering::Less
    } else if is1.0 > is2.0 {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
  }
  /*
  let iss0: Vec<Vec<(Flt, Object)>> = os.iter().map(|o| calc_distance(r, o)).collect();
  let iss1 = iss0.concat();
  */
  let mut iss1: Vec<(Flt, Object)> = vec![];
  for o in os {
    let is = calc_distance(r, o);
    for i in &is {
      if i.0 < NEARLY0 { continue; }
      iss1.push(*i);
    }
    //if is.0 > &NEARLY0 { continue; }
    //iss1.append(&mut is);
  }
  
  if iss1.len() == 0 {
    None
  } else {
//    let mut iss: Vec<(Flt, Object)> = iss1.iter().filter(|i| i.0 > NEARLY0).cloned().collect();
    iss1.sort_by(|a, b| sorting(a, b));
    let (t, obj) = iss1.first().unwrap();
    let p = r.target(*t);
    let nvec = obj.shape.get_normal(&p);
    if let Some(mut n) = nvec {
      if n.dot(&r.dir) > 0.0 {
        n = -n;
        Some(Intersection {pos: p, nvec: n, mate: obj.material, io: InOut::Out})
      } else {
        Some(Intersection {pos: p, nvec: n, mate: obj.material, io: InOut::In})
      }
    } else {
      None
    }
  }
}

fn calc_distance(r: &Ray, o: &Object) -> Vec<(Flt, Object)> {
  let ts = o.shape.distance(r);
  let mut iss: Vec<(Flt, Object)> = vec![];
  for t in &ts {
    iss.push((*t, *o))
  }
  iss
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_filter() {
    let r = 0.1 * 0.1;
    let wt1 = filter_cone(&0.0, &r);
    assert_eq!(wt1, 2.538461538461538);
    let wt2 = filter_cone(&r, &r);
    assert_eq!(wt2, 0.23076923076923078);
    let wt3 = filter_gauss(&0.0, &r);
    assert_eq!(wt3, 1.2730000000000001);
    let wt4 = filter_gauss(&r, &r);
    assert_eq!(wt4, 0.6061526928041553);
  }
}





