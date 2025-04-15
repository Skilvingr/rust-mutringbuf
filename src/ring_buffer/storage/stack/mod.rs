#![cfg_attr(doc, doc(cfg(not(feature = "vmem"))))]
#![cfg(any(not(feature = "vmem"), doc))]

pub mod rb;

use core::ops::Index;

use crate::ring_buffer::storage::Storage;
use crate::{MutRB, UnsafeSyncCell};
use crate::iterators::{ConsIter, ProdIter, WorkIter};

/// Stack-allocated storage.
pub struct StackStorage<T, const N: usize> {
    inner: [UnsafeSyncCell<T>; N]
}

impl<T, const N: usize> From<[T; N]> for StackStorage<T, N> {
    fn from(value: [T; N]) -> StackStorage<T, N> {
        let value = core::mem::ManuallyDrop::new(value);
        let ptr = &value as *const _ as *const [UnsafeSyncCell<T>; N];

        Self {
            inner: unsafe { ptr.read() }
        }
    }
}

impl<T, const N: usize> From<[UnsafeSyncCell<T>; N]> for StackStorage<T, N> {
    fn from(value: [UnsafeSyncCell<T>; N]) -> StackStorage<T, N> {
        Self {
            inner: value
        }
    }
}

impl<T, const N: usize> Index<usize> for StackStorage<T, N> {
    type Output = UnsafeSyncCell<T>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.inner.get_unchecked(index) }
    }
}

impl<T, const N: usize> Storage for StackStorage<T, N> {
    type Item = T;

    #[inline]
    fn as_ptr(&self) -> *const Self::Output {
        self.inner.as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut Self::Output {
        self.inner.as_mut_ptr()
    }

    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Trait needed to call `split` method on a stack-allocated buffer.
pub trait StackSplit<B: MutRB> {
    /// Borrows the buffer, yielding two iterators. See:
    /// - [`ProdIter`];
    /// - [`ConsIter`].
    fn split(&mut self) -> (ProdIter<B>, ConsIter<B, false>);

    /// Borrows the buffer, yielding three iterators. See:
    /// - [`ProdIter`];
    /// - [`WorkIter`];
    /// - [`ConsIter`].
    fn split_mut(&mut self) -> (ProdIter<B>, WorkIter<B>, ConsIter<B, true>);
}