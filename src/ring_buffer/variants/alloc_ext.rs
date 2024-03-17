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

impl<T> ConcurrentHeapRB<T> {
    /// Creates a new concurrent heap-allocated buffer with given capacity and uninitialised elements.
    #[allow(clippy::uninit_vec)] // handled later by UnsafeSyncCell
    pub fn new(capacity: usize) -> Self {
        let mut v = Vec::with_capacity(capacity);
        unsafe { v.set_len(capacity); }
        Self::from(v)
    }

    /// Creates a new concurrent heap-allocated buffer with given capacity and elements initialised to `default`.
    pub fn default(capacity: usize) -> Self
        where T: Default + Clone {
        Self::from(vec![T::default(); capacity])
    }
}


// Local

impl<T> From<Vec<T>> for LocalHeapRB<T> {
    fn from(value: Vec<T>) -> Self {
        Self::_from(HeapStorage::from(value))
    }
}

impl<T> LocalHeapRB<T> {
    /// Creates a new local heap-allocated buffer with given capacity.
    #[allow(clippy::uninit_vec)] // handled later by UnsafeSyncCell
    pub fn new(capacity: usize) -> Self {
        let mut v = Vec::with_capacity(capacity);
        unsafe { v.set_len(capacity); }
        Self::from(v)
    }

    /// Creates a new local heap-allocated buffer with given capacity and elements initialised to `default`.
    pub fn default(capacity: usize) -> Self
        where T: Default + Clone {
        Self::from(vec![T::default(); capacity])
    }
}