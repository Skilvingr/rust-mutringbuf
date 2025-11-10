use core::cell::UnsafeCell;
use core::num::NonZeroUsize;
use core::sync::atomic::AtomicU8;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::{Acquire, Release};

use crate::iterators::{ConsIter, ProdIter, WorkIter};
use crate::ring_buffer::storage::Storage;
use crate::ring_buffer::variants::ring_buffer_trait::{
    ConcurrentRB, IterManager, MutRB, PrivateIterManager, StorageManager,
};
use crate::ring_buffer::wrappers::buf_ref::BufRef;
use crossbeam_utils::CachePadded;

use crate::ring_buffer::storage::impl_splits::impl_splits;
#[cfg(feature = "alloc")]
use crate::{HeapSplit, HeapStorage};
#[cfg(not(feature = "vmem"))]
use crate::{StackSplit, StackStorage};

/// Concurrent mutable ring buffer. This buffer is useful for implementing types.
/// For more direct usage, consider using one of the following alternatives:
/// - [`crate::ConcurrentHeapRB`]
/// - [`crate::ConcurrentStackRB`].
pub struct ConcurrentMutRingBuf<S: Storage> {
    inner_len: NonZeroUsize,
    inner: UnsafeCell<S>,

    prod_idx: CachePadded<AtomicUsize>,
    work_idx: CachePadded<AtomicUsize>,
    cons_idx: CachePadded<AtomicUsize>,

    alive_iters: AtomicU8,
}

impl<S: Storage<Item = T>, T> MutRB for ConcurrentMutRingBuf<S> {
    type Item = T;
}

impl<S: Storage<Item = T>, T> ConcurrentRB for ConcurrentMutRingBuf<S> {}

impl_splits!(ConcurrentMutRingBuf);

impl<S: Storage<Item = T>, T> ConcurrentMutRingBuf<S> {
    pub(crate) fn _from(value: S) -> ConcurrentMutRingBuf<S> {
        assert!(value.len() > 0);

        ConcurrentMutRingBuf {
            inner_len: NonZeroUsize::new(value.len()).unwrap(),
            inner: value.into(),

            prod_idx: CachePadded::new(0.into()),
            work_idx: CachePadded::new(0.into()),
            cons_idx: CachePadded::new(0.into()),
            alive_iters: AtomicU8::default(),
        }
    }
}

impl<S: Storage> PrivateIterManager for ConcurrentMutRingBuf<S> {
    fn set_alive_iters(&self, count: u8) {
        self.alive_iters.store(count, Release);
    }

    #[inline(always)]
    fn drop_iter(&self) -> u8 {
        self.alive_iters.fetch_sub(1, Release)
    }

    #[inline(always)]
    fn acquire_fence(&self) {
        #[cfg(not(feature = "thread_sanitiser"))]
        core::sync::atomic::fence(Acquire);

        // ThreadSanitizer does not support memory fences. To avoid false positive
        // reports use atomic loads for synchronization instead.
        #[cfg(feature = "thread_sanitiser")]
        self.alive_iters.load(Acquire);
    }
}

impl<S: Storage> IterManager for ConcurrentMutRingBuf<S> {
    #[inline]
    fn prod_index(&self) -> usize {
        self.prod_idx.load(Acquire)
    }

    #[inline]
    fn work_index(&self) -> usize {
        self.work_idx.load(Acquire)
    }

    #[inline]
    fn cons_index(&self) -> usize {
        self.cons_idx.load(Acquire)
    }

    #[inline]
    fn set_prod_index(&self, index: usize) {
        self.prod_idx.store(index, Release);
    }

    #[inline]
    fn set_work_index(&self, index: usize) {
        self.work_idx.store(index, Release);
    }

    #[inline]
    fn set_cons_index(&self, index: usize) {
        self.cons_idx.store(index, Release);
    }

    fn alive_iters(&self) -> u8 {
        self.alive_iters.load(Acquire)
    }
}

impl<S: Storage<Item = T>, T> StorageManager for ConcurrentMutRingBuf<S> {
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
