// material


use super::physics::*;
use super::optics::*;

type Flt = f64;

#[derive(Debug)]
pub struct Material {
  emittance: Radiance,
  reflectance: Color,
  transmittance: Color,
  specular_refl: Color,
  ior:           Color,
  diffuseness:   Flt,
  metalness:     Flt,
  smoothness:    Flt,
}

impl Material {
  pub fn average_ior(&self) -> Flt {
    (self.ior.0 + self.ior.1 + self.ior.2) / 3.0
  }
}

/*
#[cfg(test)]


mod tests {
  use super::*;

  #[test]
  fn test_material() {
    let mat = Material();
  }

}

*/

