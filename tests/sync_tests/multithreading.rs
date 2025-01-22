extern crate alloc;

use crate::ConcurrentStackRB;
use mutringbuf::{ConcurrentHeapRB, HeapSplit, MRBIterator, StackSplit};
use std::thread;
use std::time::Instant;

const BUFFER_SIZE: usize = 300;

macro_rules! get_prod {
    ($s: ident, $prod: ident) => {
        $s.spawn(move || {
            let start = Instant::now();

            while start.elapsed().as_secs() < 3 {
                $prod.push_slice(&[0; 30]);
            }
        })
    };
}
macro_rules! get_work {
    ($s: ident, $work: ident) => {
        $s.spawn(move || {
            let start = Instant::now();

            while start.elapsed().as_secs() < 3 {
                let avail = $work.available();
                unsafe { $work.advance(avail); }
            }
        })
    };
}
macro_rules! get_cons {
    ($s: ident, $cons: ident) => {
        $s.spawn(move || {
            let start = Instant::now();

            while start.elapsed().as_secs() < 3 {
                let avail = $cons.available();
                unsafe { $cons.advance(avail); }
            }
        })
    };
}

#[test]
fn test_mt_non_workable() {
    let (mut prod, mut cons) = ConcurrentHeapRB::default(BUFFER_SIZE).split();

    let mut buf = ConcurrentStackRB::<u32, BUFFER_SIZE>::default();


    thread::scope(|s| {
        let producer = get_prod!(s, prod);
        let consumer = get_cons!(s, cons);

        let _ = producer.join();
        let _ = consumer.join();


        let (mut prod, mut cons) = buf.split();

        let producer = get_prod!(s, prod);
        let consumer = get_cons!(s, cons);

        let _ = producer.join();
        let _ = consumer.join();
    });
}

#[test]
fn test_mt_workable() {
    let (mut prod, mut work, mut cons) = ConcurrentHeapRB::default(BUFFER_SIZE).split_mut();

    let mut buf = ConcurrentStackRB::<u32, BUFFER_SIZE>::default();

    thread::scope(|s| {
        let producer = get_prod!(s, prod);
        let worker = get_work!(s, work);
        let consumer = get_cons!(s, cons);

        let _ = producer.join();
        let _ = worker.join();
        let _ = consumer.join();


        let (mut prod, mut work, mut cons) = buf.split_mut();

        let producer = get_prod!(s, prod);
        let worker = get_work!(s, work);
        let consumer = get_cons!(s, cons);

        let _ = producer.join();
        let _ = worker.join();
        let _ = consumer.join();
    });
}
