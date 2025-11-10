#![cfg_attr(doc, doc(cfg(feature = "async")))]
#![cfg(any(feature = "async", doc))]

use core::cell::UnsafeCell;
use core::num::NonZeroUsize;
use core::sync::atomic::AtomicU8;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::{Acquire, Release};

use crate::iterators::{
    AsyncConsIter, AsyncProdIter, AsyncWorkIter, async_iterators::AsyncIterator,
};
use crate::iterators::{ConsIter, ProdIter, WorkIter};
use crate::ring_buffer::storage::Storage;
use crate::ring_buffer::variants::ring_buffer_trait::{
    ConcurrentRB, IterManager, MutRB, PrivateIterManager, StorageManager,
};
use crate::ring_buffer::wrappers::buf_ref::BufRef;
use crossbeam_utils::CachePadded;
use futures::task::AtomicWaker;

/// Concurrent and asynchronous mutable ring buffer. This buffer is useful for implementing types.
/// For more direct usage, consider using one of the following alternatives:
/// - [`crate::AsyncHeapRB`]
/// - [`crate::AsyncStackRB`].
pub struct AsyncMutRingBuf<S: Storage> {
    inner_len: NonZeroUsize,
    inner: UnsafeCell<S>,

    prod_idx: CachePadded<AtomicUsize>,
    work_idx: CachePadded<AtomicUsize>,
    cons_idx: CachePadded<AtomicUsize>,

    pub(crate) prod_waker: CachePadded<AtomicWaker>,
    pub(crate) work_waker: CachePadded<AtomicWaker>,
    pub(crate) cons_waker: CachePadded<AtomicWaker>,

    alive_iters: AtomicU8,
}

impl<S: Storage<Item = T>, T> MutRB for AsyncMutRingBuf<S> {
    type Item = T;
}

impl<S: Storage<Item = T>, T> ConcurrentRB for AsyncMutRingBuf<S> {}

impl<'buf, S: Storage<Item = T> + 'buf, T> AsyncMutRingBuf<S> {
    /// Consumes the buffer, yielding three async iterators. See:
    /// - [`AsyncProdIter`];
    /// - [`AsyncWorkIter`];
    /// - [`AsyncConsIter`].
    #[cfg(any(feature = "alloc", doc))]
    pub fn split_mut(
        self,
    ) -> (
        AsyncProdIter<'buf, S, true>,
        AsyncWorkIter<'buf, S>,
        AsyncConsIter<'buf, S, true>,
    ) {
        self.set_alive_iters(3);

        let r = BufRef::new(self);
        (
            AsyncProdIter::from_sync(ProdIter::new(r.clone()), r.clone()),
            AsyncWorkIter::from_sync(WorkIter::new(r.clone()), r.clone()),
            AsyncConsIter::from_sync(ConsIter::new(r.clone()), r),
        )
    }

    /// Consumes the buffer, yielding two async iterators. See:
    /// - [`AsyncProdIter`];
    /// - [`AsyncConsIter`].
    #[cfg(any(not(feature = "alloc"), doc))]
    pub fn split(&mut self) -> (AsyncProdIter<S, false>, AsyncConsIter<S, false>) {
        self.set_alive_iters(2);

        let r = BufRef::from_ref(self);
        (
            AsyncProdIter::from_sync(ProdIter::new(r.clone())),
            AsyncConsIter::from_sync(ConsIter::new(r)),
        )
    }

    /// Consumes the buffer, yielding three async iterators. See:
    /// - [`AsyncProdIter`];
    /// - [`AsyncWorkIter`];
    /// - [`AsyncConsIter`].
    #[cfg(any(not(feature = "alloc"), doc))]
    pub fn split_mut(
        &mut self,
    ) -> (
        AsyncProdIter<S, true>,
        AsyncWorkIter<S>,
        AsyncConsIter<S, true>,
    ) {
        self.set_alive_iters(3);

        let r = BufRef::from_ref(self);
        (
            AsyncProdIter::from_sync(ProdIter::new(r.clone())),
            AsyncWorkIter::from_sync(WorkIter::new(r.clone())),
            AsyncConsIter::from_sync(ConsIter::new(r)),
        )
    }

    /// Consumes the buffer, yielding two async iterators. See:
    /// - [`AsyncProdIter`];
    /// - [`AsyncConsIter`].
    #[cfg(any(feature = "alloc", doc))]
    pub fn split(self) -> (AsyncProdIter<'buf, S, false>, AsyncConsIter<'buf, S, false>) {
        self.set_alive_iters(2);

        let r = BufRef::new(self);
        (
            AsyncProdIter::from_sync(ProdIter::new(r.clone()), r.clone()),
            AsyncConsIter::from_sync(ConsIter::new(r.clone()), r),
        )
    }

    pub(crate) fn _from(value: S) -> AsyncMutRingBuf<S> {
        assert!(value.len() > 0);

        AsyncMutRingBuf {
            inner_len: NonZeroUsize::new(value.len()).unwrap(),
            inner: value.into(),

            prod_idx: CachePadded::new(0.into()),
            work_idx: CachePadded::new(0.into()),
            cons_idx: CachePadded::new(0.into()),

            prod_waker: CachePadded::new(AtomicWaker::new()),
            work_waker: CachePadded::new(AtomicWaker::new()),
            cons_waker: CachePadded::new(AtomicWaker::new()),

            alive_iters: AtomicU8::default(),
        }
    }
}

impl<S: Storage> PrivateIterManager for AsyncMutRingBuf<S> {
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

impl<S: Storage> IterManager for AsyncMutRingBuf<S> {
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

impl<S: Storage<Item = T>, T> StorageManager for AsyncMutRingBuf<S> {
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
