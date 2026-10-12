// ray module

use std::f64;

pub mod algebra;
pub mod geometry;
pub mod light;
//pub mod mapper;
pub mod material;
pub mod object;
pub mod optics;
pub mod surface;
pub mod utils;

pub type Flt = f64;

pub const NEARLY0: f64 = 0.0001;        // 100 micro meter
pub const PI:      f64 = f64::consts::PI;
pub const PI2:     f64 = f64::consts::PI * 2.0; 
pub const PI4:     f64 = f64::consts::PI * 4.0;
pub const PI_2ND:  f64 = f64::consts::PI / 2.0;
pub const SQ_PI2:  f64 = 2.0 * f64::consts::PI * f64::consts::PI; // π x steradian (2π) for half sphere
pub const ONE_PI:  f64 = 1.0 / f64::consts::PI;                   // one over π (integral of hemisphere)
pub const SR_HALF: f64 = 1.0 / PI2;                               // half of steradian
