use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::thread;
use std::time::Duration;
use mutringbuf::ConcurrentHeapRB;

const RB_SIZE: usize = 10;

fn f() {
    let buf = ConcurrentHeapRB::from(vec![0; RB_SIZE]);
    let (mut prod, mut cons) = buf.split();

    // Flag variable to stop threads
    let stop = Arc::new(AtomicBool::new(false));

    let stop_clone = stop.clone();
    // An infinite stream of data
    let producer = thread::spawn(move || {
        let mut produced = vec![];
        let mut counter = 0;

        while !stop_clone.load(Acquire) {
            if prod.push(counter).is_ok() {

                // Store produced values to check them later
                produced.push(counter);

                // Reset counter to avoid overflow
                if counter < u8::MAX { counter += 1; } else { counter = 0; }
            }
        }

        // Iterator has to be returned here, as it was moved at the beginning of the thread
        (prod, produced)
    });

    let stop_clone = stop.clone();
    let consumer = thread::spawn(move || {
        let mut consumed = vec![];

        while !stop_clone.load(Acquire) {
            // Store consumed values to check them later
            if let Some(value) = cons.pop() { consumed.push(value); }
        }

        // Iterator has to be returned here, as it was moved at the beginning of the thread
        (cons, consumed)
    });

    // Let threads run for a while...
    thread::sleep(Duration::from_millis(1));
    // Stop threads
    stop.store(true, Release);

    let (_prod, produced) = producer.join().unwrap();
    let (mut cons, mut consumed) = consumer.join().unwrap();

    // Consume the remaining part of the buffer
    if let Some((head, tail)) = cons.peek_available() {
        consumed.extend_from_slice(head);
        consumed.extend_from_slice(tail);
    }

    assert_eq!(produced, consumed)
}

fn main() {
   for _ in 0 .. 1000 {
       f();
   }
}