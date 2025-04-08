use std::hint::black_box;
use iai_callgrind::{library_benchmark, library_benchmark_group};
use mutringbuf::{ConcurrentStackRB, LocalStackRB, StackSplit};
use iai_callgrind::main;


const RB_SIZE: usize = 256;
const BATCH_SIZE: usize = 100;


#[library_benchmark]
#[bench::long(1000)]
pub fn push_pop_local(value: u64) {
    let mut buf = LocalStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]);

    for _ in 0..value {
        prod.push(1).unwrap();
        black_box(cons.pop().unwrap());
    }
}

#[library_benchmark]
#[bench::long(1000)]
pub fn push_pop_shared(value: u64) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]);

    for _ in 0..value {
        prod.push(1).unwrap();
        black_box(cons.pop().unwrap());
    }
}

#[library_benchmark]
#[bench::long(1000)]
pub fn push_pop_x100_local(value: u64) {
    let mut buf = LocalStackRB::<u64, {RB_SIZE}>::default();

    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]).unwrap();

    for _ in 0..value {
        for _ in 0..BATCH_SIZE {
            prod.push(1).unwrap();
        }
        for _ in 0..BATCH_SIZE {
            black_box(cons.pop().unwrap());
        }
    }
}

#[library_benchmark]
#[bench::long(1000)]
pub fn push_pop_x100(value: u64) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();

    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]).unwrap();

    for _ in 0..value {
        for _ in 0..BATCH_SIZE {
            prod.push(1).unwrap();
        }
        for _ in 0..BATCH_SIZE {
            black_box(cons.pop().unwrap());
        }
    }
}

#[library_benchmark]
#[bench::long(1000)]
fn slice_x10(value: u64) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]);

    let mut data = [1; 10];
    for _ in 0..value {
        prod.push_slice(&data);
        cons.copy_slice(&mut data);
        black_box(data);
    }
}

#[library_benchmark]
#[bench::long(1000)]
fn slice_x100(value: u64) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]);

    let mut data = [1; 100];
    for _ in 0..value {
        prod.push_slice(&data);
        cons.copy_slice(&mut data);
        black_box(data);
    }
}


library_benchmark_group!(
    name = bench_iai_base;
    benchmarks = push_pop_local, push_pop_shared, push_pop_x100_local, push_pop_x100, slice_x10, slice_x100
);

main!(library_benchmark_groups = bench_iai_base);
