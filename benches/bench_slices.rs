#![cfg(bench)]
use criterion::{criterion_group, criterion_main};

mod benchmarks;

criterion_group!(bench_slices, benchmarks::slice::setup_slices);
criterion_main!(bench_slices);