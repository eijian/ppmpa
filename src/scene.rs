// scene

use super::ray::algebra::*;
use super::ray::geometry::*;
use super::ray::light::*;
use super::ray::material::*;
use super::ray::object::*;
use super::ray::optics::*;
use super::ray::physics::*;

// CONSTANTS
pub const M_AIR: Material = Material {
  emittance:     Radiance::RADIANCE0,
  reflectance:   Color::WHITE,
  transmittance: Color::WHITE,
  specular_refl: Color::BLACK,
  ior:           Color(1.0, 1.0, 1.0),
  diffuseness:   0.0,
  metalness:     0.0,
  smoothness:    0.0,
};

pub fn read_scene(file: &str) -> (Vec<Light>, Vec<Object>) {

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
    reflectance: Color(0.5, 0.5, 0.5),
    transmittance: Color(0.0, 0.0, 0.0),
    specular_refl: Color(0.8, 0.8, 0.8),
    ior:           Color(0.0, 0.0, 0.0),
    diffuseness:   1.0,
    metalness:     0.0,
    smoothness:    0.0,
  };
  let mwallb = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    reflectance: Color(0.1, 0.1, 0.4),
    transmittance: Color(0.0, 0.0, 0.0),
    specular_refl: Color(0.8, 0.0, 0.8),
    ior:           Color(0.0, 0.0, 0.0),
    diffuseness:   1.0,
    metalness:     0.0,
    smoothness:    0.0,
  };
  let mwallr = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    reflectance: Color(0.4, 0.1, 0.1),
    transmittance: Color(0.0, 0.0, 0.0),
    specular_refl: Color(0.0, 0.0, 0.0),
    ior:           Color(0.0, 0.0, 0.0),
    diffuseness:   1.0,
    metalness:     0.0,
    smoothness:    0.0,
  };
  let glass = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    reflectance: Color(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    specular_refl: Color(0.08, 0.08, 0.08),
    ior:           Color(2.0, 2.0, 2.0),
    diffuseness:   0.0,
    metalness:     0.0,
    smoothness:    0.0,
  };
  let silver = Material {
    emittance: Radiance(0.0, 0.0, 0.0),
    reflectance: Color(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    specular_refl: Color(0.78, 0.78, 0.78),
    ior:           Color(0.0, 0.0, 0.0),
    diffuseness:   0.0,
    metalness:     1.0,
    smoothness:    0.0,
  };
  let mparal = Material {
    emittance: Radiance(0.7958, 0.7958, 0.7958),
    reflectance: Color(0.0, 0.0, 0.0),
    transmittance: Color(0.0, 0.0, 0.0),
    specular_refl: Color(0.8, 0.8, 0.8),
    ior:           Color(0.0, 0.0, 0.0),
    diffuseness:   0.0,
    metalness:     0.0,
    smoothness:    0.0,
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
  let ball_glass = Object {
    shape: Shape::Sphere {center: Position3::new_pos(1.0, 0.7, 2.6), radius: 0.7},
    material: glass
  };
  let ball_mirror = Object {
    shape: Shape::Sphere {center: Position3::new_pos(-0.9, 0.7, 3.8), radius: 0.7},
    material: silver
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
         ball_glass, ball_mirror, ceiling_light])
}
