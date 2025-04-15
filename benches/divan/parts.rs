use divan::black_box;
use mutringbuf::{HeapSplit, ConcurrentHeapRB, MRBIterator,};

const BUFFER_SIZE: usize = 4096;

fn main() {
    divan::main();
}

#[divan::bench(sample_size = 100000)]
fn advance(b: divan::Bencher) {
    let buf = ConcurrentHeapRB::default(BUFFER_SIZE);
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; BUFFER_SIZE / 2]);

    b.bench_local(|| {
        unsafe { prod.advance(1); }
        unsafe { cons.advance(1); }
    });
}

#[divan::bench(sample_size = 100000)]
fn available(b: divan::Bencher) {
    let buf = ConcurrentHeapRB::default(BUFFER_SIZE);
    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[0; BUFFER_SIZE / 4]);
    cons.reset_index();
    prod.push_slice(&[1; BUFFER_SIZE / 2]);

    b.bench_local(|| {
        black_box(prod.available());
        black_box(&mut prod);
    });
}