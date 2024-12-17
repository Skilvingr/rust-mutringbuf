#![cfg(feature = "alloc")]

use mutringbuf::{ConcurrentHeapRB, ConcurrentStackRB};

pub mod async_tests;
pub mod sync_tests;
pub mod stack;

#[test]
#[should_panic]
fn len_zero_heap() {
    let _ = ConcurrentHeapRB::<i32>::default(0);
}

#[test]
#[should_panic]
fn len_zero_stack() {
    let _ = ConcurrentStackRB::<i32, 0>::default();
}