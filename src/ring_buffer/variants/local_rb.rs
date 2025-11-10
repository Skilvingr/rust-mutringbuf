use core::cell::UnsafeCell;
use core::num::NonZeroUsize;

use crate::iterators::{ConsIter, ProdIter, WorkIter};
use crate::ring_buffer::storage::Storage;
use crate::ring_buffer::variants::ring_buffer_trait::{
    IterManager, MutRB, PrivateIterManager, StorageManager,
};
use crate::ring_buffer::wrappers::buf_ref::BufRef;
#[cfg(feature = "alloc")]
use crate::{HeapSplit, HeapStorage};
#[cfg(not(feature = "vmem"))]
use crate::{StackSplit, StackStorage};

use crate::ring_buffer::storage::impl_splits::impl_splits;

/// Local (non-concurrent) mutable ring buffer. This buffer is useful for implementing types.
/// For more direct usage, consider using one of the following alternatives:
/// - [`crate::LocalHeapRB`]
/// - [`crate::LocalStackRB`].
pub struct LocalMutRingBuf<S: Storage> {
    inner_len: NonZeroUsize,
    inner: UnsafeCell<S>,

    prod_idx: UnsafeCell<usize>,
    work_idx: UnsafeCell<usize>,
    cons_idx: UnsafeCell<usize>,

    alive_iters: UnsafeCell<u8>,
}

impl<S: Storage<Item = T>, T> MutRB for LocalMutRingBuf<S> {
    type Item = T;
}

impl_splits!(LocalMutRingBuf);

impl<S: Storage<Item = T>, T> LocalMutRingBuf<S> {
    pub(crate) fn _from(value: S) -> LocalMutRingBuf<S> {
        assert!(value.len() > 0);

        LocalMutRingBuf {
            inner_len: NonZeroUsize::new(value.len()).unwrap(),
            inner: value.into(),

            prod_idx: 0.into(),
            work_idx: 0.into(),
            cons_idx: 0.into(),

            alive_iters: 0.into(),
        }
    }
}

impl<S: Storage> PrivateIterManager for LocalMutRingBuf<S> {
    fn set_alive_iters(&self, count: u8) {
        unsafe {
            *self.alive_iters.get() = count;
        }
    }

    fn drop_iter(&self) -> u8 {
        unsafe {
            let ret = *self.alive_iters.get();
            *self.alive_iters.get() -= 1;
            ret
        }
    }

    fn acquire_fence(&self) {}
}

impl<S: Storage> IterManager for LocalMutRingBuf<S> {
    #[inline]
    fn prod_index(&self) -> usize {
        unsafe { *self.prod_idx.get() }
    }

    #[inline]
    fn work_index(&self) -> usize {
        unsafe { *self.work_idx.get() }
    }

    #[inline]
    fn cons_index(&self) -> usize {
        unsafe { *self.cons_idx.get() }
    }

    #[inline]
    fn set_prod_index(&self, index: usize) {
        unsafe {
            *self.prod_idx.get() = index;
        }
    }

    #[inline]
    fn set_work_index(&self, index: usize) {
        unsafe {
            *self.work_idx.get() = index;
        }
    }

    #[inline]
    fn set_cons_index(&self, index: usize) {
        unsafe {
            *self.cons_idx.get() = index;
        }
    }

    fn alive_iters(&self) -> u8 {
        unsafe { *self.alive_iters.get() }
    }
}

impl<S: Storage<Item = T>, T> StorageManager for LocalMutRingBuf<S> {
    type StoredType = T;
    type S = S;

    #[inline]
    fn inner(&self) -> &S {
        unsafe { &(*self.inner.get()) }
    }

    #[inline]
    fn inner_mut(&self) -> &mut S {
        unsafe { &mut (*self.inner.get()) }
    }

    #[inline]
    fn inner_len(&self) -> usize {
        self.inner_len.get()
    }
}
