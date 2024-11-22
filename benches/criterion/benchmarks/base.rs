#![allow(dead_code)]


use criterion::{Bencher, black_box, Criterion};
use mutringbuf::{ConcurrentStackRB, MRBIterator, StackSplit, LocalStackRB};

const RB_SIZE: usize = 256;
const BATCH_SIZE: usize = 100;

pub fn setup(c: &mut Criterion) {
    c.bench_function("push_pop_local", push_pop_local);
    c.bench_function("push_pop_shared", push_pop_shared);
    c.bench_function("push_pop_x100_local", push_pop_x100_local);
    c.bench_function("push_pop_x100", push_pop_x100);
    c.bench_function("push_pop_work", push_pop_work);
}

fn push_pop_local(b: &mut Bencher) {
    let mut buf = LocalStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]);

    b.iter(|| {
        prod.push(1).unwrap();
        black_box(cons.pop().unwrap());
    });
}

fn push_pop_shared(b: &mut Bencher) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]);

    b.iter(|| {
        prod.push(1).unwrap();
        black_box(cons.pop().unwrap());
    });
}

fn push_pop_x100(b: &mut Bencher) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();

    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]);

    b.iter(|| {
        for _ in 0..BATCH_SIZE {
            prod.push(1).unwrap();
        }
        for _ in 0..BATCH_SIZE {
            black_box(cons.pop().unwrap());
        }
    });
}

fn push_pop_x100_local(b: &mut Bencher) {
    let mut buf = LocalStackRB::<u64, {RB_SIZE}>::default();

    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]);

    b.iter(|| {
        for _ in 0..BATCH_SIZE {
            prod.push(1).unwrap();
        }
        for _ in 0..BATCH_SIZE {
            black_box(cons.pop().unwrap());
        }
    });
}

fn push_pop_work(b: &mut Bencher) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut work, mut cons) = buf.split_mut();

    #[inline]
    fn f(x: &mut u64) {
        *x += 1u64;
    }

    for _ in 0..RB_SIZE / 2 {
        prod.push(1).unwrap();
    }

    b.iter(|| {
        for _ in 0..BATCH_SIZE {
            prod.push(1).unwrap();
        }

        for _ in 0..BATCH_SIZE {
            if let Some(data) = work.get_workable() {
                f(data);
                unsafe { work.advance(1) };
            }
        }
        for _ in 0..BATCH_SIZE {
            black_box(cons.pop().unwrap());
        }
    });
}