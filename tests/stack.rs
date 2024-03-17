use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::atomic::Ordering::{Acquire, Release};
use std::thread;
use std::time::Duration;
use mutringbuf::{ConcurrentStackRB, Iterator as MRBIt, LocalStackRB};

const BUFFER_SIZE: usize = 300;

#[test]
fn test_local_stack() {
    let (mut prod, mut work, mut cons) = LocalStackRB::<usize, { BUFFER_SIZE + 1 }>::default()
        .split_mut(0);

    assert_eq!(prod.available(), BUFFER_SIZE);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    for i in 0..BUFFER_SIZE {
        let _ = prod.push(i);
    }
    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), BUFFER_SIZE);
    assert_eq!(cons.available(), 0);

    for _ in 0..BUFFER_SIZE {
        if let Some((data, _)) = work.get_workable() {
            *data += 1;
            unsafe { work.advance(1) };
        }
    }
    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), BUFFER_SIZE);

    for i in 0..BUFFER_SIZE {
        assert_eq!(cons.pop().unwrap(), i + 1);
    }

    assert_eq!(prod.available(), BUFFER_SIZE);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);
}



const RB_SIZE: usize = 30;

fn rb_fibonacci() {
    let buf = ConcurrentStackRB::<usize, RB_SIZE>::default();
    let (mut prod, mut work, mut cons) = buf.split_mut((1, 0));

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

        while !prod_finished_clone.load(Acquire) || work.index() != prod_last_index_clone.load(Acquire) {

            if let Some((value, (bt_h, bt_t))) = work.get_workable() {
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
            if let Some(value) = cons.pop() { consumed.push(value); }
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
fn fibonacci_test_stack() {
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