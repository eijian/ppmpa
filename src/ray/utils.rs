// utils module

use rand::Rng;

use super::*;


// Russian Roulette
pub fn russian_roulette(cs: &[Flt]) -> usize {
  let mut rng = rand::thread_rng();
  let p: Flt = rng.gen_range(0.0, 1.0);
  check_under(cs, p)
}

#[inline]
fn check_under(cs: &[Flt], p: Flt) -> usize {
  let mut i: usize = 0;
  while i < cs.len() && p > cs[i] {
    i += 1;
  }
  i
}

// Russian Roulette Binary
//   true: under or equal of c
//   false: over of c
pub fn russian_roulette_binary(c: Flt) -> bool {
  if c <= 0.0 {
    false
  } else {
    let mut rng = rand::thread_rng();
    let p: Flt = rng.gen_range(0.0, 1.0);
    check_under_binary(c, p)
  }
}

#[inline]
fn check_under_binary(c: Flt, p: Flt) -> bool {
    c > 0.0 && p <= c
}

// density_power
//   IN:
//     r   冪乗パラメータ (-1.0 <= r <= 1.0)
//   OUT:
//     n   減衰量  10^(6*r)
//         r=-1 -> 0
//         r=0  -> 1
//         r=1  -> 10^6 (≒ ♾️)
//     1/(n+1)

pub fn density_power(r: Flt) -> (Flt, Flt) {
  let r2 = if r > 1.0 {
    1.0
  } else {
    if r < -1.0 {
      -1.0
    } else {
      r
    }
  };
  //let n = (10.0 as Flt).powf(6.0 * r2);
  let n = if r2 == -1.0 { 0.0 } else { (10.0 as Flt).powf(10.0 * r2) };
  (n, 1.0 / (n + 1.0))
}

// TESTS

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_support_func() {
    // russian_roulette
    assert_eq!(check_under(&[0.0], 0.0), 0);
    assert_eq!(check_under(&[0.0], 0.5), 1);
    assert_eq!(check_under(&[0.0], 1.0), 1);
    assert_eq!(check_under(&[1.0], 0.0), 0);
    assert_eq!(check_under(&[1.0], 0.5), 0);
    assert_eq!(check_under(&[1.0], 1.0), 0);
    assert_eq!(check_under(&[0.5], 0.0), 0);
    assert_eq!(check_under(&[0.5], 0.5), 0);
    assert_eq!(check_under(&[0.5], 0.5001), 1);
    assert_eq!(check_under(&[0.5], 1.0), 1);

    // russian_roulette_binary
    assert_eq!(russian_roulette_binary(0.0), false);
    assert_eq!(russian_roulette_binary(-1.0), false);
    assert_eq!(russian_roulette_binary(1.0), true);
    assert_eq!(russian_roulette_binary(2.0), true);
    assert_eq!(check_under_binary(0.0, 0.0), false);
    assert_eq!(check_under_binary(0.0, 0.00001), false);
    assert_eq!(check_under_binary(1.0, 1.0), true);
    assert_eq!(check_under_binary(1.0, 0.99999), true);
    assert_eq!(check_under_binary(0.5, 0.5), true);
    assert_eq!(check_under_binary(0.5, 0.50001), false);


    // density_power
  }
}



