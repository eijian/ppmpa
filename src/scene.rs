// scene

use super::ray::algebra::*;
use super::ray::geometry::*;
use super::ray::light::*;
use super::ray::material::*;
use super::ray::object::*;
use super::ray::optics::*;
use super::ray::physics::*;
use super::ray::surface::*;

// CONSTANTS
pub const M_AIR: Material = Material {
  emittance:     Radiance::RADIANCE0,
  transmittance: Color::WHITE,
  ior:           Color(1.0, 1.0, 1.0),
  surface: Surface::Nothing,
};

pub fn read_scene(_file: &str) -> (Vec<Light>, Vec<Object>) {

  // light
  let l1 = Light::ParallelogramLight {
    color: Color(1.0, 1.0, 1.0).normalize(),
    flux: 5.0,
    pos: Vector3::new(-0.67, 3.99, 2.33),
    nvec: -Vector3::EY,
    dir1: Vector3::new(1.33, 0.0, 0.0),
    dir2: Vector3::new(0.0, 0.0, 1.33),
  };

  // material
  let mwall = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    surface: Surface::new_simple(
      &Color(0.5, 0.5, 0.5),
      &Color(0.8, 0.8, 0.8),
      &1.0,
      &0.0,
      &0.0,
    ),
  };
  let mwallb = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    surface: Surface::new_simple(
      &Color(0.1, 0.1, 0.4),
      &Color(0.8, 0.0, 0.8),
      &1.0,
      &0.0,
      &0.0,
    ),
  };
  let mwallr = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    surface: Surface::new_simple(
      &Color(0.4, 0.1, 0.1),
      &Color(0.0, 0.0, 0.0),
      &1.0,
      &0.0,
      &0.0,
    ),
  };
  // == ORIGINAL
  /*
  let glass = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(2.0, 2.0, 2.0),
    surface: Surface::Simple {
      reflectance: Color(0.0, 0.0, 0.0),
      specular_refl: Color(0.08, 0.08, 0.08),
      diffuseness:   0.0,
      metalness:     0.0,
      roughness:    0.0,
    },
  };
  */
  /*
  let glass = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    reflectance: Color(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    specular_refl: Color(0.08, 0.08, 0.08),
    ior:           Color(2.0, 2.0, 2.0),
    diffuseness:   0.0,
    metalness:     0.0,
    roughness:    0.0,
  };
  */
  // == ORIGINAL
  /*
  let silver = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    surface: Surface::Simple {
      reflectance: Color(0.0, 0.0, 0.0),
      specular_refl: Color(0.78, 0.78, 0.78),
      diffuseness:   0.0,
      metalness:     1.0,
      roughness:    0.0,
    },
  };
  */
  /*
  let silver = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    reflectance: Color(0.5, 0.3, 0.1),
    transmittance: Color(0.0, 0.0, 0.0),
    specular_refl: Color(0.78, 0.78, 0.78),
    ior:           Color(0.0, 0.0, 0.0),
    diffuseness:   0.0,
    metalness:     1.0,
    roughness:    0.0,
  };
  */
  /*
  let ball10 = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    //surface: Surface::new_simple(
    surface: Surface::new_ts(
      &Color(0.0, 0.0, 0.0),
      &Color(0.78, 0.78, 0.78),
      &0.0,
      &1.0,
      &1.0,
    ),
  };
  */
  let ball10 = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(1.5, 1.5, 1.5),
    /*
    surface: Surface::new_simple(
      &Color(1.0, 1.0, 1.0),
      &Color(0.05, 0.05, 0.05),
      &0.0,
      &0.0,
      &0.0,
    ),
    */
    surface: Surface::new_ts(
      //&Color(0.6, 0.35, 0.1),
      //&Color(0.05, 0.05, 0.05),
      &Color(1.0, 1.0, 1.0),
      &Color(0.05, 0.05, 0.05),
      &0.0,
      &0.0,
      &0.8,
    ),
    
  };

  let ball9 = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    surface: Surface::new_simple(
      &Color(0.0, 0.0, 0.0),
      &Color(0.78, 0.78, 0.78),
      &0.0,
      &1.0,
      &0.9,
    ),
  };
  let ball8 = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    surface: Surface::new_simple(
      &Color(0.0, 0.0, 0.0),
      &Color(0.78, 0.78, 0.78),
      &0.0,
      &1.0,
      &0.8,
    ),
  };
  let ball7 = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    surface: Surface::new_simple(
      &Color(0.0, 0.0, 0.0),
      &Color(0.78, 0.78, 0.78),
      &0.0,
      &1.0,
      &0.7,
    ),
  };
  /*
  let ball6 = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    surface: Surface::new_simple(
      &Color(0.0, 0.0, 0.0),
      &Color(0.78, 0.78, 0.78),
      &0.0,
      &1.0,
      &0.6,
    ),
  };
  */
  let ball6 = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    surface: Surface::new_ts(
      &Color(0.0, 0.0, 0.0),
      &Color(0.78, 0.78, 0.78),
      &0.0,
      &1.0,
      &0.6,
    ),
  };

  let ball5 = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    surface: Surface::new_ts(
      &Color(0.0, 0.0, 0.0),
      &Color(0.78, 0.78, 0.78),
      &0.0,
      &1.0,
      &0.5,
    ),
  };
  /*
  let ball5 = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    //surface: Surface::new_simple(
    surface: Surface::new_ts(
      &Color(0.6, 0.35, 0.1),
      &Color(0.05, 0.05, 0.05),
      &1.0,
      &0.0,
      &0.9,
    ),
  };
  */
  /*
  let ball5 = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    //ior:           Color(0.0, 0.0, 0.0),
    ior:           Color(2.0, 2.0, 2.0),
    /*
    surface: Surface::new_simple(
      &Color(0.6, 0.35, 0.1),
      &Color(0.08, 0.08, 0.08),
      &0.5,
      &0.0,
      &0.5,
    ),
    */
    surface: Surface::new_ts(
      //&Color(0.6, 0.35, 0.1),
      //&Color(0.05, 0.05, 0.05),
      &Color(1.0, 1.0, 1.0),
      &Color(0.05, 0.05, 0.05),
      &0.0,
      &0.0,
      &0.0,
    ),
  };
  */
  let ball4 = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    surface: Surface::new_simple(
      &Color(0.0, 0.0, 0.0),
      &Color(0.78, 0.78, 0.78),
      &0.0,
      &1.0,
      &0.4,
    ),
  };
  let ball3 = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    surface: Surface::new_simple(
      &Color(0.0, 0.0, 0.0),
      &Color(0.78, 0.78, 0.78),
      &0.0,
      &1.0,
      &0.3,
    ),
  };
  let ball2 = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    surface: Surface::new_simple(
      &Color(0.0, 0.0, 0.0),
      &Color(0.78, 0.78, 0.78),
      &0.0,
      &1.0,
      &0.2,
    ),
  };
  /*
  let ball1 = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    //surface: Surface::new_simple(
    surface: Surface::new_ts(
      &Color(0.0, 0.0, 0.0),
      &Color(0.78, 0.78, 0.78),
      &0.0,
      &1.0,
      &0.1,
    ),
  };
  */
  let ball1 = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(1.5, 1.5, 1.5),
    //surface: Surface::new_simple(
    surface: Surface::new_ts(
      &Color(0.6, 0.35, 0.1),
      &Color(0.05, 0.05, 0.05),
      &1.0,
      &0.0,
      &0.0,
    ),
  };


  let mparal = Material {
    //emittance: Radiance(0.7958, 0.7958, 0.7958),
    emittance: Radiance(0.15, 0.15, 0.15),
    transmittance: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    surface: Surface::new_simple(
      &Color(0.0, 0.0, 0.0),
      &Color(0.8, 0.8, 0.8),
      &0.0,
      &0.0,
      &0.0,
    ),
  };

  // objects
  let flooring = Object {
    shape: Shape::Plain {nvec: Vector3::EY, dist: 0.0},
    material: mwall
  };
  let ceiling  = Object {
    shape: Shape::Plain {nvec: -Vector3::EY, dist: 4.0},
    material: mwall
  };
  let rsidewall = Object {
    shape: Shape::Plain {nvec: -Vector3::EX, dist: 2.0},
    material: mwallb
  };
  let lsidewall = Object {
    shape: Shape::Plain {nvec: Vector3::EX, dist: 2.0},
    material: mwallr
  };
  let backwall = Object {
    shape: Shape::Plain {nvec: Vector3::EZ, dist: 6.0},
    material: mwall
  };
  let frontwall = Object {
    shape: Shape::Plain {nvec: -Vector3::EZ, dist: 5.0},
    material: mwall
  };
  /*
  let ball_glass = Object {
    shape: Shape::Sphere {center: Position3::new_pos(1.0, 0.7, 2.6), radius: 0.7},
    material: glass
  };
  let ball_mirror = Object {
    shape: Shape::Sphere {center: Position3::new_pos(-0.9, 0.7, 3.8), radius: 0.7},
    material: silver
  };
  */
  let ball_1 = Object {
    shape: Shape::Sphere {center: Position3::new_pos(-1.6, 1.5, 3.0), radius: 0.4},
    material: ball1
  };
  let ball_2 = Object {
    shape: Shape::Sphere {center: Position3::new_pos(-0.8, 1.5, 3.0), radius: 0.4},
    material: ball2
  };
  let ball_3 = Object {
    shape: Shape::Sphere {center: Position3::new_pos(0.0, 1.5, 3.0), radius: 0.4},
    material: ball3
  };
  let ball_4 = Object {
    shape: Shape::Sphere {center: Position3::new_pos(0.8, 1.5, 3.0), radius: 0.4},
    material: ball4
  };
  let ball_5 = Object {
    shape: Shape::Sphere {center: Position3::new_pos(1.6, 1.5, 3.0), radius: 0.4},
    material: ball5
  };
  let ball_6 = Object {
    shape: Shape::Sphere {center: Position3::new_pos(-1.6, 0.5, 2.5), radius: 0.4},
    material: ball6
  };
  let ball_7 = Object {
    shape: Shape::Sphere {center: Position3::new_pos(-0.8, 0.5, 2.5), radius: 0.4},
    material: ball7
  };
  let ball_8 = Object {
    shape: Shape::Sphere {center: Position3::new_pos(0.0, 0.5, 2.5), radius: 0.4},
    material: ball8
  };
  let ball_9 = Object {
    shape: Shape::Sphere {center: Position3::new_pos(0.8, 0.5, 2.5), radius: 0.4},
    material: ball9
  };
  let ball_10 = Object {
    shape: Shape::Sphere {center: Position3::new_pos(1.6, 0.5, 2.5), radius: 0.4},
    material: ball10
  };
  let ceiling_light = Object {
    shape: Shape::Parallelogram {position: Vector3::new(-0.67, 3.99, 2.33),
                                 nvec: -Vector3::EY,
                                 dir1: Vector3::new(0.67, 3.99, 2.33) - Vector3::new(-0.67, 3.99, 2.33),
                                 dir2: Vector3::new(-0.67, 3.99, 3.67) - Vector3::new(-0.67, 3.99, 2.33),
    },
    material: mparal
  };
  
  (vec![l1],
   vec![flooring, ceiling, rsidewall, lsidewall, backwall, frontwall,
         //ball_glass, ball_mirror,
         ball_1, ball_2, ball_3, ball_4, ball_5, ball_6, ball_7, ball_8, ball_9, ball_10,
         ceiling_light])
}
