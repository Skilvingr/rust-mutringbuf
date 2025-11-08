use core::cell::UnsafeCell;
use core::num::NonZeroUsize;
use core::sync::atomic::Ordering::{Acquire, Release};
use core::sync::atomic::{AtomicBool, AtomicUsize};

use crate::iterators::{ConsIter, ProdIter, WorkIter};
use crate::ring_buffer::storage::Storage;
use crate::ring_buffer::variants::ring_buffer_trait::{
    ConcurrentRB, IterManager, MutRB, StorageManager,
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

    prod_alive: AtomicBool,
    work_alive: AtomicBool,
    cons_alive: AtomicBool,
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

            prod_alive: AtomicBool::default(),
            work_alive: AtomicBool::default(),
            cons_alive: AtomicBool::default(),
        }
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

    fn prod_alive(&self) -> bool {
        self.prod_alive.load(Acquire)
    }

    fn work_alive(&self) -> bool {
        self.work_alive.load(Acquire)
    }

    fn cons_alive(&self) -> bool {
        self.cons_alive.load(Acquire)
    }

    fn set_prod_alive(&self, alive: bool) {
        self.prod_alive.store(alive, Release);
    }

    fn set_work_alive(&self, alive: bool) {
        self.work_alive.store(alive, Release);
    }

    fn set_cons_alive(&self, alive: bool) {
        self.cons_alive.store(alive, Release);
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
