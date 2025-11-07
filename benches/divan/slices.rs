use divan::black_box;
use mutringbuf::{ConcurrentHeapRB, HeapSplit};

const BUFFER_SIZE: usize = 4096;

fn main() {
    divan::main();
}

#[divan::bench(sample_size = 100000)]
fn slice_x10(b: divan::Bencher) {
    let buf = ConcurrentHeapRB::default(BUFFER_SIZE);
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; BUFFER_SIZE / 2]);

    let mut data = [1; 10];
    b.bench_local(|| {
        prod.push_slice(&data);
        cons.copy_slice(&mut data);
        black_box(data);
    });
}

#[divan::bench(sample_size = 100000)]
fn slice_x100(b: divan::Bencher) {
    let buf = ConcurrentHeapRB::default(BUFFER_SIZE);
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; BUFFER_SIZE / 2]);

    let mut data = [1; 100];
    b.bench_local(|| {
        prod.push_slice(&data);
        cons.copy_slice(&mut data);
        black_box(data);
    });
}

#[divan::bench(sample_size = 100000)]
fn slice_x1000_local(b: divan::Bencher) {
    let buf = ConcurrentHeapRB::default(BUFFER_SIZE);
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; BUFFER_SIZE / 2]);

    let mut data = [1; 1000];
    b.bench_local(|| {
        prod.push_slice(&data);
        cons.copy_slice(&mut data);
        black_box(data);
    });
}

#[divan::bench(sample_size = 100000)]
fn slice_x1000(b: divan::Bencher) {
    let buf = ConcurrentHeapRB::default(BUFFER_SIZE);
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; BUFFER_SIZE / 2]);

    let mut data = [1; 1000];
    b.bench_local(|| {
        prod.push_slice(&data);
        cons.copy_slice(&mut data);
        black_box(data);
    });
}

#[divan::bench(sample_size = 100000)]
fn slice_x1000_clone(b: divan::Bencher) {
    let buf = ConcurrentHeapRB::default(BUFFER_SIZE);
    let (mut prod, mut cons) = buf.split();

    prod.push_slice_clone(&[1; BUFFER_SIZE / 2]);

    let mut data = [1; 1000];
    b.bench_local(|| {
        prod.push_slice_clone(&data);
        cons.clone_slice(&mut data);
        black_box(data);
    });
}

#[divan::bench(sample_size = 100000)]
fn slice_xbuf_size(b: divan::Bencher) {
    let buf = ConcurrentHeapRB::default(BUFFER_SIZE);
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; BUFFER_SIZE / 2]);

    let mut data = [1; BUFFER_SIZE - 1];
    b.bench_local(|| {
        prod.push_slice(&data);
        cons.copy_slice(&mut data);
        black_box(data);
    });
}
