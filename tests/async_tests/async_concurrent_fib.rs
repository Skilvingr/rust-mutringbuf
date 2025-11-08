use std::{
    sync::{
        Arc,
        atomic::{
            AtomicBool, AtomicUsize,
            Ordering::{Acquire, Release},
        },
    },
    time::Instant,
};

use mutringbuf::iterators::async_iterators::AsyncIterator;

use crate::common_def;

common_def!(buf);

async fn rb_fibonacci() {
    #[cfg(not(feature = "vmem"))]
    let buf = mutringbuf::AsyncStackRB::from([0; BUFFER_SIZE]);
    #[cfg(feature = "vmem")]
    let buf = mutringbuf::AsyncHeapRB::from(vec![0; BUFFER_SIZE]);

    let (mut as_prod, mut as_work, mut as_cons) = buf.split_mut();

    // Flag variable to stop threads
    let stop_prod = Arc::new(AtomicBool::new(false));
    let prod_finished = Arc::new(AtomicBool::new(false));
    let prod_last_index = Arc::new(AtomicUsize::new(0));

    let stop_clone = stop_prod.clone();
    let prod_last_index_clone = prod_last_index.clone();
    let prod_finished_clone = prod_finished.clone();
    // An infinite stream of data
    let mut producer = tokio::task::spawn(async move {
        let mut produced = vec![];
        let mut counter = 1usize;

        while !stop_clone.load(Acquire) {
            as_prod.push(counter).await;

            // Store produced values to check them later
            produced.push(counter);

            // Reset counter to avoid overflow
            if counter < 20 {
                counter += 1;
            } else {
                counter = 1;
            }
        }

        prod_last_index_clone.store(as_prod.index(), Release);
        prod_finished_clone.store(true, Release);

        // Iterator has to be returned here, as it was moved at the beginning of the thread
        (as_prod, produced)
    });

    let prod_last_index_clone = prod_last_index.clone();
    let prod_finished_clone = prod_finished.clone();
    let mut worker = tokio::task::spawn(async move {
        let mut acc = (1, 0);

        while !prod_finished_clone.load(Acquire)
            || as_work.index() != prod_last_index_clone.load(Acquire)
        {
            if let Some(value) = as_work.get_workable().await {
                let (bt_h, bt_t) = &mut acc;

                if *value == 1 {
                    (*bt_h, *bt_t) = (1, 0);
                }

                *value = *bt_h + *bt_t;

                (*bt_h, *bt_t) = (*bt_t, *value);

                unsafe { as_work.advance(1) };
            }
        }

        as_work
    });

    let mut consumer = tokio::task::spawn(async move {
        let mut consumed = vec![];

        while !prod_finished.load(Acquire) || as_cons.index() != prod_last_index.load(Acquire) {
            // Store consumed values to check them later
            if let Some(value) = as_cons.peek_ref().await {
                consumed.push(*value);
                unsafe {
                    as_cons.advance(1);
                }
            }
        }

        // Iterator has to be returned here, as it was moved at the beginning of the thread
        (as_cons, consumed)
    });

    let start = Instant::now();

    // advance futures
    while start.elapsed().as_millis() <= 500 {}

    // Stop producer
    stop_prod.store(true, Release);

    let (mut prod, produced) = producer.await.unwrap();
    let mut work = worker.await.unwrap();
    let (mut cons, consumed) = consumer.await.unwrap();

    assert_eq!(prod.available(), BUFFER_SIZE - 1);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);
    assert_eq!(
        consumed,
        produced.iter().map(|v| fib(*v)).collect::<Vec<usize>>()
    );

    //println!("{:?}", produced);
    //println!("{:?}", consumed);
    //println!("{:?}", produced.iter().map(|v| fib(*v)).collect::<Vec<usize>>())
}

#[tokio::test]
async fn async_fibonacci_test() {
    for _ in 0..10 {
        rb_fibonacci().await;
    }
}

pub fn fib(n: usize) -> usize {
    match n {
        1 | 2 => 1,
        3 => 2,
        _ => fib(n - 1) + fib(n - 2),
    }
}
