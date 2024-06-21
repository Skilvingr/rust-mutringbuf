#![allow(dead_code)]

extern crate alloc;

use alloc::vec;

use criterion::{Bencher, black_box, Criterion};
use mutringbuf::{ConcurrentHeapRB, ConcurrentStackRB, MRBIterator};

const RB_SIZE: usize = 256;
const BATCH_SIZE: usize = 100;

pub fn setup(c: &mut Criterion) {
    c.bench_function("push_pop_shared", push_pop_shared);
    c.bench_function("pop_x100", pop_x100);
    c.bench_function("push_pop_x100", push_pop_x100);
    c.bench_function("push_pop_work", push_pop_work);
}

fn push_pop_shared(b: &mut Bencher) {
    let buf = ConcurrentHeapRB::from(vec![0u64; RB_SIZE + 1]);
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]);

    b.iter(|| {
        prod.push(1).unwrap();
        black_box(cons.peek_ref().unwrap());
        unsafe { cons.advance(1); }
    });
}

fn pop_x100(b: &mut Bencher) {
    let buf = ConcurrentHeapRB::from(vec![0u64; RB_SIZE + 1]);
    let (mut prod, mut cons) = buf.split();

    b.iter(|| {
        prod.push_slice(&[1; BATCH_SIZE]);

        for _ in 0..BATCH_SIZE {
            black_box(cons.peek_ref().unwrap());
            unsafe { cons.advance(1); }
        }
    });
}

fn push_pop_x100(b: &mut Bencher) {
    let buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();

    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]).unwrap();

    b.iter(|| {
        for _ in 0..BATCH_SIZE {
            prod.push(1).unwrap();
        }
        for _ in 0..BATCH_SIZE {
            black_box(cons.peek_ref().unwrap());
            unsafe { cons.advance(1); }
        }
    });
}

fn push_pop_work(b: &mut Bencher) {
    let buf = ConcurrentHeapRB::from(vec![0u64; RB_SIZE + 1]);
    let (mut prod, mut work, mut cons) = buf.split_mut();

    let f = |x: &mut u64| {
        *x += 1u64;
    };

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
            black_box(cons.peek_ref().unwrap());
            unsafe { cons.advance(1); }
        }
    });
}