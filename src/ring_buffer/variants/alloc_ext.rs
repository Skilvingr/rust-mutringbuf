#![cfg(feature = "alloc")]

use crate::HeapStorage;
#[allow(unused_imports)]
use crate::ProdIter;
use crate::{ConcurrentMutRingBuf, LocalMutRingBuf, UnsafeSyncCell};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

macro_rules! impl_rb {
    ($t: tt) => {
        impl<T> From<Vec<T>> for $t<T> {
            #[doc = concat!("Converts a `Vec<T>` into a [`", stringify!($t), "`].")]
            /// Note that the length of the buffer will be equal to the length of the vector, and *not*
            /// to its capacity.
            fn from(value: Vec<T>) -> Self {
                Self::_from(HeapStorage::from(value))
            }
        }
        
        impl<T> $t<T> {
            #[doc = concat!("Creates a new [`", stringify!($t), "`] with given capacity and zeroed (uninitialised) elements.")]
            /// # Safety
            /// The buffer must be then initialised using proper [`ProdIter`] methods (`*_init` ones).
            pub unsafe fn new_zeroed(capacity: usize) -> Self {
                Self::_from(
                    HeapStorage::from(
                        (0..capacity).map(|_| UnsafeSyncCell::new_zeroed()).collect::<Box<[UnsafeSyncCell<T>]>>()
                    )
                )
            }
        
            #[doc = concat!("Creates a new [`", stringify!($t), "`] with given capacity and elements initialised to `default`.")]
            pub fn default(capacity: usize) -> Self
                where T: Default + Clone {
                Self::from(vec![T::default(); capacity])
            }
        }
    };
}

// Concurrent

/// A heap-allocated ring buffer usable in concurrent environment.
pub type ConcurrentHeapRB<T> = ConcurrentMutRingBuf<HeapStorage<T>>;

impl_rb!(ConcurrentHeapRB);


// Local

/// A heap-allocated ring buffer usable in local environment.
pub type LocalHeapRB<T> = LocalMutRingBuf<HeapStorage<T>>;

impl_rb!(LocalHeapRB);