// material


use super::physics::*;
use super::optics::*;

type Flt = f64;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Material {
  pub emittance: Radiance,
  pub reflectance: Color,
  pub transmittance: Color,
  pub specular_refl: Color,
  pub ior:           Color,
  pub diffuseness:   Flt,
  pub metalness:     Flt,
  pub smoothness:    Flt,
}

impl Material {
  pub fn average_ior(&self) -> Flt {
    (self.ior.0 + self.ior.1 + self.ior.2) / 3.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_material() {
    let mat = Material {
      emittance: Radiance::RADIANCE0,
      reflectance: Color(1.0, 0.7, 0.9),
      transmittance: Color(0.0, 0.0, 0.0),
      specular_refl: Color(0.1, 0.3, 0.5),
      ior: Color(0.8, 0.2, 0.5),
      diffuseness: 0.5,
      metalness: 0.1,
      smoothness: 0.2,
    };
    assert_eq!(mat.average_ior(), 0.5);
    assert_eq!(mat.reflectance, Color(1.0, 0.7, 0.9));
  }

}

