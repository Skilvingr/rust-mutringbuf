use divan::black_box;
use mutringbuf::{HeapSplit, LocalHeapRB, MRBIterator};

const BUFFER_SIZE: usize = 4096;
const BATCH_SIZE: usize = 100;

fn main() {
    divan::main();
}

#[divan::bench(sample_size = 100000)]
fn push_pop_local(b: divan::Bencher) {
    let buf = LocalHeapRB::default(BUFFER_SIZE);
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; BUFFER_SIZE / 2]);

    b.bench_local(|| {
        prod.push(1).unwrap();
        black_box(cons.pop().unwrap());
    });
}

#[divan::bench(sample_size = 100000)]
fn push_pop_shared(b: divan::Bencher) {
    let buf = LocalHeapRB::default(BUFFER_SIZE);
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; BUFFER_SIZE / 2]);

    b.bench_local(|| {
        prod.push(1).unwrap();
        black_box(cons.pop().unwrap());
    });
}

#[divan::bench(sample_size = 100000)]
fn push_pop_x100(b: divan::Bencher) {
    let buf = LocalHeapRB::default(BUFFER_SIZE);

    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; BUFFER_SIZE / 2]);

    b.bench_local(|| {
        for _ in 0..BATCH_SIZE {
            prod.push(1).unwrap();
        }
        for _ in 0..BATCH_SIZE {
            black_box(cons.pop().unwrap());
        }
    });
}

#[divan::bench(sample_size = 100000)]
fn push_pop_x100_local(b: divan::Bencher) {
    let buf = LocalHeapRB::default(BUFFER_SIZE);

    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; BUFFER_SIZE / 2]);

    b.bench_local(|| {
        for _ in 0..BATCH_SIZE {
            prod.push(1).unwrap();
        }
        for _ in 0..BATCH_SIZE {
            black_box(cons.pop().unwrap());
        }
    });
}

#[divan::bench(sample_size = 100000)]
fn push_pop_x100_heap(b: divan::Bencher) {
    let buf = LocalHeapRB::default(BUFFER_SIZE);

    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; BUFFER_SIZE / 2]);

    b.bench_local(|| {
        for _ in 0..BATCH_SIZE {
            prod.push(1).unwrap();
        }
        for _ in 0..BATCH_SIZE {
            black_box(cons.pop().unwrap());
        }
    });
}

#[divan::bench(sample_size = 100000)]
fn push_pop_work(b: divan::Bencher) {
    let buf = LocalHeapRB::default(BUFFER_SIZE);
    let (mut prod, mut work, mut cons) = buf.split_mut();

    #[inline]
    fn f(x: &mut u64) {
        *x += 1u64;
    }

    for _ in 0..BUFFER_SIZE / 2 {
        prod.push(1).unwrap();
    }

    b.bench_local(|| {
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
