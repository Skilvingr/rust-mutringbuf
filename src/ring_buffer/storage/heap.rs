#![cfg(feature = "alloc")]

use alloc::boxed::Box;
use alloc::vec::Vec;

use core::ops::Index;

use crate::ring_buffer::storage::storage_trait::Storage;
use crate::UnsafeSyncCell;

pub struct HeapStorage<T> {
    inner: Box<[UnsafeSyncCell<T>]>
}

impl<T> From<Box<[T]>> for HeapStorage<T> {
    fn from(value: Box<[T]>) -> Self {
        Self {
            inner: unsafe { core::mem::transmute::<Box<[T]>, Box<[UnsafeSyncCell<T>]>>(value) }
        }
    }
}

impl<T> From<Vec<T>> for HeapStorage<T> {
    fn from(value: Vec<T>) -> Self {
        Self::from(value.into_boxed_slice())
    }
}

impl<T> Index<usize> for HeapStorage<T> {
    type Output = UnsafeSyncCell<T>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<T> Storage<T> for HeapStorage<T> {
    #[inline]
    fn as_ptr(&self) -> *const Self::Output {
        self.inner.as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut Self::Output {
        self.inner.as_mut_ptr()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}