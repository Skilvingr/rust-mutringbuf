#![allow(dead_code)]


use criterion::{Bencher, black_box, Criterion};
use mutringbuf::{ConcurrentStackRB, MRBIterator, StackSplit};

const RB_SIZE: usize = 256;

pub fn setup(c: &mut Criterion) {
    c.bench_function("advance", advance);
    c.bench_function("available", available);
}

fn advance(b: &mut Bencher) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]);

    b.iter(|| {
        unsafe { prod.advance(1); }
        unsafe { cons.advance(1); }
    });
}

fn available(b: &mut Bencher) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[0; 1 * RB_SIZE / 4]);
    cons.reset_index();
    prod.push_slice(&[1; RB_SIZE / 2]);

    b.iter(|| {
        black_box(prod.available());
        black_box(&mut prod);
    });
}