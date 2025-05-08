#![cfg(feature = "alloc")]
#![allow(unused_mut)]

pub mod async_tests;
pub mod common;
pub mod sync_tests;

#[cfg(feature = "vmem")]
macro_rules! common_def {
    () => {
        use mutringbuf::HeapSplit;
        common_def!(buf);
    };
    (buf) => {
        #[cfg(target_arch = "aarch64")]
        const BUFFER_SIZE: usize = 16384;
        #[cfg(not(target_arch = "aarch64"))]
        const BUFFER_SIZE: usize = 4096;
    }
}
#[cfg(not(feature = "vmem"))]
macro_rules! common_def {
    () => {
        use mutringbuf::StackSplit;
        common_def!(buf);
    };
    (buf) => {
        const BUFFER_SIZE: usize = 400;
    }
}

#[cfg(feature = "vmem")]
macro_rules! get_buf {
    (Local) => {
        #[cfg(feature = "vmem")]
        mutringbuf::LocalHeapRB::from(vec![0; BUFFER_SIZE])
    };
    (Concurrent) => { 
        mutringbuf::ConcurrentHeapRB::from(vec![0; BUFFER_SIZE])
    }
}
#[cfg(not(feature = "vmem"))]
macro_rules! get_buf {
    (Local) => {
        mutringbuf::LocalStackRB::from([0; BUFFER_SIZE])

    };
    (Concurrent) => { 
        mutringbuf::ConcurrentStackRB::from([0; BUFFER_SIZE])
    }
}
pub(crate) use {common_def, get_buf};

#[test]
#[should_panic]
fn len_zero_heap() {
    let _ = mutringbuf::ConcurrentHeapRB::<i32>::default(0);
}

#[cfg(not(feature = "vmem"))]
#[test]
#[should_panic]
fn len_zero_stack() {
    let _ = mutringbuf::ConcurrentStackRB::<i32, 0>::default();
}
