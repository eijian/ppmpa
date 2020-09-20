// tracer

use std::f64;
use std::vec::*;
use std::cmp::*;

use super::ray::*;
use super::ray::algebra::*;
use super::ray::geometry::*;
use super::ray::object::*;
use super::ray::light::*;
use super::ray::material::*;
use super::ray::physics::*;
use super::ray::optics::*;

use super::screen::*;
use super::scene::*;

const ONE_PI: Flt  = 1.0 / f64::consts::PI;
const SR_HALF: Flt = 1.0 / (2.0 * f64::consts::PI);

// Photon tracing

pub fn trace_photon(uc: &bool, m0: &Material, objs: &Vec<Object>, l: i32, ph: &Photon) -> Vec<PhotonCache> {
  if l >= 10 {
    return vec![]
  }
  
  let is = calc_intersection(&ph.ray, objs);
  let is1 = is.unwrap();
  let d = is1.mate.diffuseness;
  let i = russian_roulette(&[d]);
  let mut pcs = if i > 0 {
    reflect_diff(uc, m0, objs, l, ph, &is1)
  } else {
    reflect_spec(uc, m0, objs, l, ph, &is1)
  };
  if (*uc == false || l > 0) && d > 0.0 {
    pcs.push(Photon::new(&ph.wl, &Ray::new(&is1.pos, &ph.ray.dir)));
  }
  pcs
}

fn reflect_diff(uc: &bool, m0: &Material, objs: &Vec<Object>, l: i32, ph: &Photon, is: &Intersection) -> Vec<PhotonCache> {
  let i = russian_roulette(&[is.mate.reflectance.select_wavelength(ph.wl)]);
  if i > 0 {
    let dr = diffuse_reflection(&is.nvec);
    trace_photon(uc, m0, objs, l+1, &Photon::new(&ph.wl, &Ray::new(&is.pos, &dr)))
  } else{
    vec![]
  }
}

fn reflect_spec(uc: &bool, m0: &Material, objs: &Vec<Object>, l: i32, ph: &Photon, is: &Intersection) -> Vec<PhotonCache> {
  let f0 = is.mate.specular_refl.select_wavelength(ph.wl);
  let (rdir, cos0) = specular_reflection(&is.nvec, &ph.ray.dir);
  let f = f0 + (1.0 - f0) * (1.0 - cos0).powf(5.0);
  let j = russian_roulette(&[f]);
  if j > 0 {
    trace_photon(uc, m0, objs, l+1, &Photon::new(&ph.wl, &Ray::new(&is.pos, &rdir)))
  } else {
    if is.mate.ior.select_wavelength(ph.wl) == 0.0 {
      vec![]
    } else {
      reflect_trans(uc, m0, objs, l, ph, is, &cos0)
    }
  }
}

fn reflect_trans(uc: &bool, m0: &Material, objs: &Vec<Object>, l: i32, ph: &Photon, is: &Intersection, c0: &Flt) -> Vec<PhotonCache> {
  let ior0 = m0.ior.select_wavelength(ph.wl);
  let ior1 = is.mate.ior.select_wavelength(ph.wl);
  let (tdir, ior2) = specular_refraction(&ior0, &ior1, c0, &ph.ray.dir, &is.nvec);
  let m02 = if tdir.dot(&is.nvec) < 0.0 {
    is.mate
  } else {
    M_AIR
  };
  trace_photon(uc, &m02, objs, l+1, &Photon::new(&ph.wl, &Ray::new(&is.pos, &tdir)))
}

// Photon mapping method

pub fn trace_ray(scr: &Screen, m0: &Material, l: i32, pmap: &PhotonMap, objs: &Vec<Object>, lgts: &Vec<Light>, r: &Ray) -> Radiance {
  if l >= 10 {
    return Radiance::RADIANCE0
  }
  let is = calc_intersection(r, objs);
  if is == None {
    return Radiance::RADIANCE0
  }

  let is1 = is.unwrap();
  let di = if scr.use_classic_for_direct == true {
    let mut rad = Radiance::RADIANCE0;
    for l in lgts {
      rad = rad + get_radiance_from_light(objs, &is1.pos, &is1.nvec, l);
    }
    rad
  } else {
    Radiance::RADIANCE0
  };
  let ii = estimate_radiance(&scr, &pmap, &is1);
  let (rdir, cos0) = specular_reflection(&is1.nvec, &r.dir);
  let df = is1.mate.diffuseness;
  let mt = is1.mate.metalness;
  let f = reflection_index(&is1.mate.specular_refl, &cos0);
  let f2 = -f;
  let ior1 = is1.mate.average_ior();
  
  let si = if df == 1.0 || f == Color::BLACK {
    Radiance::RADIANCE0
  } else {
    trace_ray(scr, m0, l+1, pmap, objs, lgts, &Ray::new(&is1.pos, &rdir))
  };
  let ti = if f2 == Color::BLACK || ior1 == 0.0 {
    Radiance::RADIANCE0
  } else {
    Radiance::RADIANCE0
  };

  is1.mate.emittance * SR_HALF + df * brdf(&is1.mate, &(di + ii)) + (1.0 - df) * (f * si + (1.0 - mt) * f2 * ti)
}

fn estimate_radiance(scr: &Screen, pmap: &PhotonMap, is: &Intersection) -> Radiance {
  Radiance::RADIANCE0
}


// CLASSIC Ray tracer

pub fn trace_ray_classic(scr: &Screen, m0: &Material, l: i32, objs: &Vec<Object>, lgts: &Vec<Light>, r: &Ray) -> Radiance {
  if l >= 10 {
    return Radiance::RADIANCE0
  }
  //return Radiance(1.0, 0.0, 0.0);
  let is = calc_intersection(r, objs);
  if is == None {
    return Radiance::RADIANCE0
  }

  let is1 = is.unwrap();
  let diffs = lgts.iter().map(|l| get_radiance_from_light(objs, &is1.pos, &is1.nvec, &l));
  let mut rad_diff = Radiance::RADIANCE0;
  for d in diffs {
    rad_diff = rad_diff + d;
  }
  let ii = Radiance::RADIANCE0;
  let (rdir, cos0) = specular_reflection(&is1.nvec, &r.dir);
  let df = is1.mate.diffuseness;
  let mt = is1.mate.metalness;
  let f = reflection_index(&is1.mate.specular_refl, &cos0);
  let f2 = -f;
  let ior1 = is1.mate.average_ior();

  let si = if  df == 1.0 || f == Color::BLACK {
    Radiance::RADIANCE0
  } else {
    trace_ray_classic(scr, m0, l+1, objs, lgts, &Ray::new(&is1.pos, &rdir))
  };
  let ti = if f2 == Color::BLACK || ior1 == 0.0 {
    Radiance::RADIANCE0
  } else {
    let ior0 = m0.average_ior();
    let (tdir, ior2) = specular_refraction(&ior0, &ior1, &cos0, &r.dir, &is1.nvec);
    let m02 = if tdir.dot(&is1.nvec) < 0.0 { is1.mate } else { M_AIR };
    trace_ray_classic(scr, &m02, l+1, objs, lgts, &Ray::new(&is1.pos, &tdir))
  };

  is1.mate.emittance * SR_HALF +
    df         * brdf(&is1.mate, &(scr.ambient + rad_diff)) +
    (1.0 - df) * (f * si + (1.0 - mt) * f2 * ti)
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
struct Intersection {
  pub pos: Position3,
  pub nvec: Direction3,
  pub mate: Material,
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
    if nvec == None {
      None
    } else {
      Some(Intersection {pos: p, nvec: nvec.unwrap(), mate: obj.material})
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

fn brdf(m: &Material, rad: &Radiance) -> Radiance {
  m.reflectance * ONE_PI * *rad
}








