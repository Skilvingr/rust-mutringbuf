#[cfg(bench)]
use criterion::{criterion_group, criterion_main};
#[cfg(bench)]
mod benchmarks;
#[cfg(bench)]
criterion_group!(benches, benchmarks::base::setup);
#[cfg(bench)]
criterion_main!(benches);

#[cfg(not(bench))]
fn main() {}