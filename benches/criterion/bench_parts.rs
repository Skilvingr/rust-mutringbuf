#[cfg(bench)]
use criterion::{criterion_group, criterion_main};
#[cfg(bench)]
mod benchmarks;
#[cfg(bench)]
criterion_group!(bench_parts, benchmarks::parts::setup);
#[cfg(bench)]
criterion_main!(bench_parts);

#[cfg(not(bench))]
fn main() {}