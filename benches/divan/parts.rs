use mutringbuf::{ConcurrentStackRB, MRBIterator, StackSplit};
use divan::black_box;

const RB_SIZE: usize = 256;

fn main() {
    divan::main();
}

#[divan::bench(sample_size = 100000)]
fn advance(b: divan::Bencher) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]);

    b.bench_local(|| {
        unsafe { prod.advance(1); }
        unsafe { cons.advance(1); }
    });
}

#[divan::bench(sample_size = 100000)]
fn available(b: divan::Bencher) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[0; RB_SIZE / 4]);
    cons.reset_index();
    prod.push_slice(&[1; RB_SIZE / 2]);

    b.bench_local(|| {
        black_box(prod.available());
        black_box(&mut prod);
    });
}