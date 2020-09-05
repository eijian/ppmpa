// ppmpa_benchmark

#[macro_use]
extern crate criterion;

use criterion::Criterion;
//use criterion::black_box;

use ppmpa::ray::algebra::*;

fn vec_add() -> Direction3 {
  let d1 = generate_random_dir();
  let d2 = generate_random_dir();
  d1 + d2
}

fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("vec3 add", |b| b.iter(|| vec_add()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

