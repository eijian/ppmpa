// object

//use super::algebra::*;
use super::geometry::*;
use super::material::*;
//use super::optics::*;
//use super::physics::*;

#[derive(Debug, Clone, Copy)]
pub struct Object {
  pub shape: Shape,
  pub material: Material,
}

impl PartialEq for Object {
  fn eq(&self, other: &Self) -> bool {
    self.shape == other.shape && self.material == other.material
  }
}

impl Object {
  pub fn new(s: &Shape, m: &Material) -> Object {
    Object {shape: *s, material: *m}
  }
}



#[cfg(test)]
mod tests {
  use super::*;
  use super::super::algebra::*;
  use super::super::optics::*;
  use super::super::physics::*;

  #[test]
  fn test_object() {
    let shp = Shape::Plain {
      nvec: Vector3::EY,
      dist: 0.0,
    };
    let mat = Material {
      emittance: Radiance::RADIANCE0,
      reflectance: Color(1.0, 0.7, 0.9),
      transmittance: Color(0.0, 0.0, 0.0),
      specular_refl: Color(0.1, 0.3, 0.5),
      ior: Color(0.8, 0.2, 0.5),
      diffuseness: 0.5,
      metalness: 0.1,
      roughness: 0.2,
    };
    let obj = Object::new(&shp, &mat);
    assert_eq!(obj.shape, shp);
    assert_eq!(obj.material, mat);
  }

}

