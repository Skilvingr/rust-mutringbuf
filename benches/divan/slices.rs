use mutringbuf::{ConcurrentStackRB, LocalStackRB, StackSplit};
use divan::black_box;

const RB_SIZE: usize = 1024;

fn main() {
    divan::main();
}

#[divan::bench(sample_size = 100000)]
fn slice_x10(b: divan::Bencher) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]);

    let mut data = [1; 10];
    b.bench_local(|| {
        prod.push_slice(&data);
        cons.copy_slice(&mut data);
        black_box(data);
    });
}

#[divan::bench(sample_size = 100000)]
fn slice_x100(b: divan::Bencher) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; RB_SIZE / 2]);

    let mut data = [1; 100];
    b.bench_local(|| {
        prod.push_slice(&data);
        cons.copy_slice(&mut data);
        black_box(data);
    });
}

#[divan::bench(sample_size = 100000)]
fn slice_x1000_local(b: divan::Bencher) {
    let mut buf = LocalStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; 12]);

    let mut data = [1; 1000];
    b.bench_local(|| {
        prod.push_slice(&data);
        cons.copy_slice(&mut data);
    });

    black_box(data);
}

#[divan::bench(sample_size = 100000)]
fn slice_x1000(b: divan::Bencher) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; 12]);

    let mut data = [1; 1000];
    b.bench_local(|| {
        prod.push_slice(&data);
        cons.copy_slice(&mut data);
    });

    black_box(data);
}

#[divan::bench(sample_size = 100000)]
fn slice_x1000_clone(b: divan::Bencher) {
    let mut buf = ConcurrentStackRB::<u64, {RB_SIZE}>::default();
    let (mut prod, mut cons) = buf.split();

    prod.push_slice_clone(&[1; 12]);

    let mut data = [1; 1000];
    b.bench_local(|| {
        prod.push_slice_clone(&data);
        cons.clone_slice(&mut data);
    });

    black_box(data);
}
