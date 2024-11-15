use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::atomic::Ordering::{Acquire, Release};
use std::time::Instant;

use mutringbuf::ConcurrentStackRB;
use mutringbuf::iterators::async_iterators::AsyncIterator;

const RB_SIZE: usize = 30;

fn rb_fibonacci() {
    let buf = ConcurrentStackRB::from([0; RB_SIZE]);
    let (
        mut as_prod,
        mut as_work,
        mut as_cons
    ) = buf.split_mut_async();


    // Flag variable to stop threads
    let stop_prod = Arc::new(AtomicBool::new(false));
    let prod_finished = Arc::new(AtomicBool::new(false));
    let prod_last_index = Arc::new(AtomicUsize::new(0));

    let stop_clone = stop_prod.clone();
    let prod_last_index_clone = prod_last_index.clone();
    let prod_finished_clone = prod_finished.clone();
    // An infinite stream of data
    let mut producer = tokio_test::task::spawn(async {
        let mut produced = vec![];
        let mut counter = 1usize;

        while !stop_clone.load(Acquire) {
            as_prod.push(counter).await;

            // Store produced values to check them later
            produced.push(counter);

            // Reset counter to avoid overflow
            if counter < 20 { counter += 1; } else { counter = 1; }
        }

        prod_last_index_clone.store(as_prod.index(), Release);
        prod_finished_clone.store(true, Release);

        // Iterator has to be returned here, as it was moved at the beginning of the thread
        (as_prod, produced)
    });

    let prod_last_index_clone = prod_last_index.clone();
    let prod_finished_clone = prod_finished.clone();
    let mut worker = tokio_test::task::spawn(async {

        let mut acc = (1, 0);

        while !prod_finished_clone.load(Acquire) || as_work.index() != prod_last_index_clone.load(Acquire) {

            if let Some(value) = as_work.get_workable().await {
                let (bt_h, bt_t) = &mut acc;

                if *value == 1 { (*bt_h, *bt_t) = (1, 0); }

                *value = *bt_h + *bt_t;

                (*bt_h, *bt_t) = (*bt_t, *value);

                unsafe { as_work.advance(1) };
            }
        }

        as_work
    });

    let mut consumer = tokio_test::task::spawn(async {
        let mut consumed = vec![];

        while !prod_finished.load(Acquire) || as_cons.index() != prod_last_index.load(Acquire) {

            // Store consumed values to check them later
            if let Some(value) = as_cons.peek_ref().await {
                consumed.push(*value);
                unsafe { as_cons.advance(1); }
            }
        }

        // Iterator has to be returned here, as it was moved at the beginning of the thread
        (as_cons, consumed)
    });

    tokio_test::block_on(async {
        let start = Instant::now();

        // advance futures
        while start.elapsed().as_millis() <= 100 {
            let _ = producer.poll();
            let _ = worker.poll();
            let _ = consumer.poll();
        }

        // Stop producer
        stop_prod.store(true, Release);

        let (mut prod, produced) = producer.await;
        let mut work = worker.await;
        let (mut cons, consumed) = consumer.await;

        assert_eq!(prod.available(), RB_SIZE - 1);
        assert_eq!(work.available(), 0);
        assert_eq!(cons.available(), 0);
        assert_eq!(consumed, produced.iter().map(|v| fib(*v)).collect::<Vec<usize>>());

        //println!("{:?}", produced);
        //println!("{:?}", consumed);
        //println!("{:?}", produced.iter().map(|v| fib(*v)).collect::<Vec<usize>>())
    });
}

#[test]
fn async_fibonacci_test() {
    for _ in 0 .. 10 {
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