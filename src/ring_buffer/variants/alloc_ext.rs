#![cfg(feature = "alloc")]
use alloc::vec;
use alloc::vec::Vec;
use crate::{ConcurrentHeapRB, LocalHeapRB};
use crate::HeapStorage;

// Concurrent

impl<T> From<Vec<T>> for ConcurrentHeapRB<T> {
    fn from(value: Vec<T>) -> Self {
        Self::_from(HeapStorage::from(value))
    }
}

impl<T: Default + Clone> ConcurrentHeapRB<T> {
    /// Creates a new heap-allocated buffer with given capacity.
    pub fn new_heap(capacity: usize) -> Self {
        Self::from(vec![T::default(); capacity])
    }
}


// Local

impl<T> From<Vec<T>> for LocalHeapRB<T> {
    fn from(value: Vec<T>) -> Self {
        Self::_from(HeapStorage::from(value))
    }
}

impl<T: Default + Clone> LocalHeapRB<T> {
    /// Creates a new heap-allocated buffer with given capacity.
    pub fn new_heap(capacity: usize) -> Self {
        Self::from(vec![T::default(); capacity])
    }
}