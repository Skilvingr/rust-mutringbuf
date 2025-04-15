#![cfg(feature = "alloc")]

pub mod rb;
pub mod vmem_helper;

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::Index;
use crate::ring_buffer::storage::Storage;
use crate::{MutRB, UnsafeSyncCell};
use crate::iterators::{ConsIter, ProdIter, WorkIter};

/// Heap-allocated storage.
pub struct HeapStorage<T> {
    inner: *mut UnsafeSyncCell<T>,
    len: usize,
}

impl <T> Drop for HeapStorage<T> {
    fn drop(&mut self) {
        unsafe {
            #[cfg(feature = "vmem")]
            libc::munmap(self.inner as _, 2 * self.len * size_of::<T>());

            #[cfg(not(feature = "vmem"))]
            let _ = Box::from_raw(core::ptr::slice_from_raw_parts_mut(self.inner, self.len));
        }
    }
}

impl<T> HeapStorage<T> {
    #[cfg(feature = "vmem")]
    fn new(value: Box<[UnsafeSyncCell<T>]>) -> Self {
        let r = vmem_helper::new(&value);

        Self {
            inner: r,
            len: value.len(),
        }
    }

    #[cfg(not(feature = "vmem"))]
    fn new(value: Box<[UnsafeSyncCell<T>]>) -> Self {
        let len = value.len();

        let v = Box::into_raw(value);

        unsafe {
            Self {
                inner: (*v).as_mut_ptr(),
                len
            }
        }
    }
}

impl<T> From<Box<[T]>> for HeapStorage<T> {
    fn from(value: Box<[T]>) -> Self {
        Self::new(unsafe { core::mem::transmute::<Box<[T]>, Box<[UnsafeSyncCell<T>]>>(value) })
    }
}

impl<T> From<Box<[UnsafeSyncCell<T>]>> for HeapStorage<T> {
    fn from(value: Box<[UnsafeSyncCell<T>]>) -> Self {
        Self::new(value)
    }
}

impl<T> From<Vec<T>> for HeapStorage<T> {
    fn from(value: Vec<T>) -> Self {
        Self::from(value.into_boxed_slice())
    }
}

impl<T> From<Vec<UnsafeSyncCell<T>>> for HeapStorage<T> {
    fn from(value: Vec<UnsafeSyncCell<T>>) -> Self {
        Self::new(value.into_boxed_slice())
    }
}

impl<T> Index<usize> for HeapStorage<T> {
    type Output = UnsafeSyncCell<T>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.inner.add(index) }
    }
}

impl<T> Storage for HeapStorage<T> {
    type Item = T;

    #[inline]
    fn as_ptr(&self) -> *const Self::Output {
        self.inner as _
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut Self::Output {
        self.inner
    }

    fn len(&self) -> usize {
        self.len
    }
}

/// Trait needed to call `split` method on a heap-allocated buffer.
#[cfg(feature = "alloc")]
pub trait HeapSplit<B: MutRB> {
    /// Consumes the buffer, yielding two iterators. See:
    /// - [`ProdIter`];
    /// - [`ConsIter`].
    fn split<'buf>(self) -> (ProdIter<'buf, B>, ConsIter<'buf, B, false>);

    /// Consumes the buffer, yielding three iterators. See:
    /// - [`ProdIter`];
    /// - [`WorkIter`];
    /// - [`ConsIter`].
    fn split_mut<'buf>(self) -> (ProdIter<'buf, B>, WorkIter<'buf, B>, ConsIter<'buf, B, true>);
}