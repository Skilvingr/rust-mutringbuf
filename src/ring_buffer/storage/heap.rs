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

impl<T> From<Box<[UnsafeSyncCell<T>]>> for HeapStorage<T> {
    fn from(value: Box<[UnsafeSyncCell<T>]>) -> Self {
        Self {
            inner: value
        }
    }
}

impl<T> From<Vec<T>> for HeapStorage<T> {
    fn from(value: Vec<T>) -> Self {
        Self::from(value.into_boxed_slice())
    }
}

impl<T> From<Vec<UnsafeSyncCell<T>>> for HeapStorage<T> {
    fn from(value: Vec<UnsafeSyncCell<T>>) -> Self {
        Self {
            inner: value.into_boxed_slice()
        }
    }
}

impl<T> Index<usize> for HeapStorage<T> {
    type Output = UnsafeSyncCell<T>;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.inner.get_unchecked(index) }
    }
}

impl<T> Storage for HeapStorage<T> {
    type Item = T;

    #[inline(always)]
    fn as_ptr(&self) -> *const Self::Output {
        self.inner.as_ptr()
    }

    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut Self::Output {
        self.inner.as_mut_ptr()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}