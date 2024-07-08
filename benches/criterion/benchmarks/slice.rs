#![allow(dead_code)]

use criterion::{Bencher, black_box, Criterion};
use mutringbuf::{ConcurrentStackRB, LocalStackRB, StackSplit};

const RB_SIZE: usize = 1024;

pub fn setup_slices(c: &mut Criterion) {
    c.bench_function("slice_x10", slice_x10);
    c.bench_function("slice_x100", slice_x100);
    c.bench_function("slice_x1000_local", slice_x1000_local);
    c.bench_function("slice_x1000", slice_x1000);
    c.bench_function("slice_x1000_clone", slice_x1000_clone);
}

fn slice_x10(b: &mut Bencher) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]);

    let mut data = [1; 10];
    b.iter(|| {
        prod.push_slice(&data);
        cons.copy_slice(&mut data);
        black_box(data);
    });
}

fn slice_x100(b: &mut Bencher) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]);

    let mut data = [1; 100];
    b.iter(|| {
        prod.push_slice(&data);
        cons.copy_slice(&mut data);
        black_box(data);
    });
}

fn slice_x1000_local(b: &mut Bencher) {
    let mut buf = LocalStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; 12]);

    let mut data = [1; 1000];
    b.iter(|| {
        prod.push_slice(&data);
        cons.copy_slice(&mut data);
    });

    black_box(data);
}

fn slice_x1000(b: &mut Bencher) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; 12]);

    let mut data = [1; 1000];
    b.iter(|| {
        prod.push_slice(&data);
        cons.copy_slice(&mut data);
    });

    black_box(data);
}

fn slice_x1000_clone(b: &mut Bencher) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice_clone(&[1; 12]);

    let mut data = [1; 1000];
    b.iter(|| {
        prod.push_slice_clone(&data);
        cons.clone_slice(&mut data);
    });

    black_box(data);
}
