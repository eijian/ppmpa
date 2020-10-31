

use std::f64;

pub mod algebra;
pub mod geometry;
pub mod material;
pub mod light;
pub mod object;
pub mod optics;
pub mod physics;
pub mod surface;

pub type Flt = f64;

pub const NEARLY0: f64 = 0.0001;        // 100 micro meter
pub const PI2:     f64 = f64::consts::PI * 2.0; 
pub const PI4:     f64 = f64::consts::PI * 4.0; 



