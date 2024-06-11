use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::atomic::Ordering::{Acquire, Release};
use std::thread;
use std::time::Duration;

use mutringbuf::{ConcurrentHeapRB, MRBIterator};

const RB_SIZE: usize = 30;

fn rb_fibonacci() {
    let buf = ConcurrentHeapRB::from(vec![0; RB_SIZE]);
    let (mut prod, mut work, mut cons) = buf.split_mut();

    // Flag variable to stop threads
    let stop_prod = Arc::new(AtomicBool::new(false));
    let prod_finished = Arc::new(AtomicBool::new(false));
    let prod_last_index = Arc::new(AtomicUsize::new(0));

    let stop_clone = stop_prod.clone();
    let prod_last_index_clone = prod_last_index.clone();
    let prod_finished_clone = prod_finished.clone();
    // An infinite stream of data
    let producer = thread::spawn(move || {
        let mut produced = vec![];
        let mut counter = 1usize;

        while !stop_clone.load(Acquire) {
            while prod.push(counter).is_err() {}

            // Store produced values to check them later
            produced.push(counter);

            // Reset counter to avoid overflow
            if counter < 20 { counter += 1; } else { counter = 1; }
        }

        prod_last_index_clone.store(prod.index(), Release);
        prod_finished_clone.store(true, Release);

        // Iterator has to be returned here, as it was moved at the beginning of the thread
        (prod, produced)
    });

    let prod_last_index_clone = prod_last_index.clone();
    let prod_finished_clone = prod_finished.clone();
    let worker = thread::spawn(move || {

        let mut acc = (1, 0);

        while !prod_finished_clone.load(Acquire) || work.index() != prod_last_index_clone.load(Acquire) {
            
            if let Some(value) = work.get_workable() {
                let (bt_h, bt_t) = &mut acc;

                if *value == 1 { (*bt_h, *bt_t) = (1, 0); }

                *value = *bt_h + *bt_t;

                (*bt_h, *bt_t) = (*bt_t, *value);

                unsafe { work.advance(1) };
            }
        }

        work
    });

    let consumer = thread::spawn(move || {
        let mut consumed = vec![];

        while !prod_finished.load(Acquire) || cons.index() != prod_last_index.load(Acquire) {

            // Store consumed values to check them later
            if let Some(value) = cons.peek_ref() {
                consumed.push(*value);
                unsafe { cons.advance(1); }
            }
        }

        // Iterator has to be returned here, as it was moved at the beginning of the thread
        (cons, consumed)
    });

    // Let threads run for a while...
    thread::sleep(Duration::from_millis(1));
    // Stop producer
    stop_prod.store(true, Release);

    let (mut prod, produced) = producer.join().unwrap();
    let mut work = worker.join().unwrap();
    let (mut cons, consumed) = consumer.join().unwrap();

    assert_eq!(prod.available(), RB_SIZE - 1);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);
    assert_eq!(consumed, produced.iter().map(|v| fib(*v)).collect::<Vec<usize>>());

    // println!("{:?}", produced);
    // println!("{:?}", consumed);
    // println!("{:?}", produced.iter().map(|v| fib(*v)).collect::<Vec<usize>>())
}

#[test]
fn fibonacci_test() {
    for _ in 0 .. 100 {
        rb_fibonacci();
    }
}

pub fn fib(n: usize) -> usize {
    match n {
        1 | 2 => 1,
        3 => 2,
        _ => fib(n - 1) + fib(n - 2),
    }
}