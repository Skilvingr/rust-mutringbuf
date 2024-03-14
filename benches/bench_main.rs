#![cfg(bench)]
use criterion::{criterion_group, criterion_main};

mod benchmarks;

criterion_group!(benches, benchmarks::base::setup);
criterion_main!(benches);