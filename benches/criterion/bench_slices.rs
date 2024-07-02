#[cfg(bench)]
use criterion::{criterion_group, criterion_main};
#[cfg(bench)]
mod benchmarks;
#[cfg(bench)]
criterion_group!(bench_slices, benchmarks::slice::setup_slices);
#[cfg(bench)]
criterion_main!(bench_slices);

#[cfg(not(bench))]
fn main() {}