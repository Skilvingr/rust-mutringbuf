#![cfg(feature = "alloc")]

use alloc::vec;
use alloc::vec::Vec;
use crate::{ConcurrentMutRingBuf, LocalMutRingBuf, UnsafeSyncCell};
use crate::HeapStorage;
#[allow(unused_imports)]
use crate::ProdIter;

// Concurrent

/// A heap-allocated ring buffer usable in concurrent environment.
pub type ConcurrentHeapRB<T> = ConcurrentMutRingBuf<HeapStorage<T>>;

impl<T> From<Vec<T>> for ConcurrentHeapRB<T> {
    /// Constructs a `ConcurrentHeapRB` using the passed vector.
    /// Note that the length of the buffer will be equal to the length of teh vector, and *not*
    /// to its capacity.
    fn from(mut value: Vec<T>) -> Self {
        value.shrink_to_fit();
        Self::_from(HeapStorage::from(value))
    }
}

impl<T> ConcurrentHeapRB<T> {
    /// Creates a new concurrent heap-allocated buffer with given capacity and zeroed (uninitialised) elements.
    /// # Safety
    /// The buffer must be then initialised using proper [`ProdIter`] methods (`*_init` ones).
    pub unsafe fn new_zeroed(capacity: usize) -> Self {
        let mut v = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            v.push(UnsafeSyncCell::new_zeroed());
        }

        Self::_from(HeapStorage::from(v))
    }

    /// Creates a new concurrent heap-allocated buffer with given capacity and elements initialised to `default`.
    pub fn default(capacity: usize) -> Self
        where T: Default + Clone {
        Self::from(vec![T::default(); capacity])
    }
}


// Local

/// A heap-allocated ring buffer usable in local environment.
pub type LocalHeapRB<T> = LocalMutRingBuf<HeapStorage<T>>;

impl<T> From<Vec<T>> for LocalHeapRB<T> {
    /// Constructs a `LocalHeapRB` using the passed vector.
    /// Note that the length of the buffer will be equal to the length of teh vector, and *not*
    /// to its capacity.
    fn from(mut value: Vec<T>) -> Self {
        value.shrink_to_fit();
        Self::_from(HeapStorage::from(value))
    }
}

impl<T> LocalHeapRB<T> {
    /// Creates a new local heap-allocated buffer with given capacity and zeroed (uninitialised) elements.
    /// # Safety
    /// The buffer must be then initialised using proper [`ProdIter`] methods (`*_init` ones).
    pub unsafe fn new_zeroed(capacity: usize) -> Self {
        let mut v = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            v.push(UnsafeSyncCell::new_zeroed());
        }

        Self::_from(HeapStorage::from(v))
    }

    /// Creates a new local heap-allocated buffer with given capacity and elements initialised to `default`.
    pub fn default(capacity: usize) -> Self
        where T: Default + Clone {
        Self::from(vec![T::default(); capacity])
    }
}