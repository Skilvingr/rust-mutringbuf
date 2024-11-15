use core::cell::UnsafeCell;
use core::num::NonZeroUsize;
use core::sync::atomic::{AtomicBool, AtomicUsize};
use core::sync::atomic::Ordering::{Acquire, Release};

use crossbeam_utils::CachePadded;

use crate::{ConcurrentStackRB, ConsIter, ProdIter, WorkIter};
#[cfg(any(feature = "async", doc))]
use crate::{AsyncConsIter, AsyncProdIter, AsyncWorkIter, iterators::async_iterators::AsyncIterator};
#[cfg(feature = "alloc")]
use crate::HeapStorage;
use crate::ring_buffer::storage::stack::StackStorage;
use crate::ring_buffer::storage::storage_trait::Storage;
#[cfg(feature = "alloc")]
use crate::ring_buffer::variants::HeapSplit;
use crate::ring_buffer::variants::impl_splits::impl_splits;
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, IterManager, MutRB, StorageManager};
use crate::ring_buffer::variants::StackSplit;
use crate::ring_buffer::wrappers::buf_ref::BufRef;

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
    /// Consumes the buffer, yielding three async iterators. See:
    /// - [`AsyncProdIter`];
    /// - [`AsyncWorkIter`];
    /// - [`AsyncConsIter`].
    #[cfg(all(feature = "alloc", any(feature = "async", doc)))]
    pub fn split_mut_async<'buf>(self) -> (AsyncProdIter<'buf, ConcurrentMutRingBuf<S>>, AsyncWorkIter<'buf, ConcurrentMutRingBuf<S>>, AsyncConsIter<'buf, ConcurrentMutRingBuf<S>, true>) {
        self.set_prod_alive(true);
        self.set_work_alive(true);
        self.set_cons_alive(true);

        let r = BufRef::new(self);
        (
            AsyncProdIter::from_sync(ProdIter::new(r.clone())),
            AsyncWorkIter::from_sync(WorkIter::new(r.clone())),
            AsyncConsIter::from_sync(ConsIter::new(r)),
        )
    }

    /// Consumes the buffer, yielding two async iterators. See:
    /// - [`AsyncProdIter`];
    /// - [`AsyncConsIter`].
    #[cfg(all(not(feature = "alloc"), any(feature = "async", doc)))]
    pub fn split_async(&mut self) -> (AsyncProdIter<ConcurrentMutRingBuf<S>>, AsyncConsIter<ConcurrentMutRingBuf<S>, false>) {
        self.set_prod_alive(true);
        self.set_cons_alive(true);

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
    #[cfg(all(not(feature = "alloc"), any(feature = "async", doc)))]
    pub fn split_mut_async(&mut self) -> (AsyncProdIter<ConcurrentMutRingBuf<S>>, AsyncWorkIter<ConcurrentMutRingBuf<S>>, AsyncConsIter<ConcurrentMutRingBuf<S>, true>) {
        self.set_prod_alive(true);
        self.set_work_alive(true);
        self.set_cons_alive(true);

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
    #[cfg(all(feature = "alloc", any(feature = "async", doc)))]
    pub fn split_async<'buf>(self) -> (AsyncProdIter<'buf, ConcurrentMutRingBuf<S>>, AsyncConsIter<'buf, ConcurrentMutRingBuf<S>, false>) {
        self.set_prod_alive(true);
        self.set_cons_alive(true);

        let r = BufRef::new(self);
        (
            AsyncProdIter::from_sync(ProdIter::new(r.clone())),
            AsyncConsIter::from_sync(ConsIter::new(r)),
        )
    }
    
    pub(crate) fn _from(value: S) -> ConcurrentMutRingBuf<S> {
        ConcurrentMutRingBuf {
            inner_len: NonZeroUsize::new(value.len()).unwrap(),
            inner: value.into(),

            prod_idx: CachePadded::new(0.into()),
            work_idx: CachePadded::new(0.into()),
            cons_idx: CachePadded::new(0.into()),

            prod_alive: AtomicBool::default(),
            work_alive: AtomicBool::default(),
            cons_alive: AtomicBool::default()
        }
    }
}

impl<S: Storage> IterManager for ConcurrentMutRingBuf<S> {
    #[inline(always)]
    fn prod_index(&self) -> usize {
        self.prod_idx.load(Acquire)
    }

    #[inline(always)]
    fn work_index(&self) -> usize {
        self.work_idx.load(Acquire)
    }

    #[inline(always)]
    fn cons_index(&self) -> usize {
        self.cons_idx.load(Acquire)
    }

    #[inline(always)]
    fn set_prod_index(&self, index: usize) {
        self.prod_idx.store(index, Release);
    }

    #[inline(always)]
    fn set_work_index(&self, index: usize) {
        self.work_idx.store(index, Release);
    }

    #[inline(always)]
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

    #[inline(always)]
    fn inner(&self) -> &S {
        unsafe { &(*self.inner.get()) }
    }

    #[inline(always)]
    fn inner_mut(&self) -> &mut S {
        unsafe { &mut (*self.inner.get()) }
    }

    #[inline(always)]
    fn inner_len(&self) -> usize {
        self.inner_len.get()
    }
}

impl<T, const N: usize> From<[T; N]> for ConcurrentStackRB<T, N> {
    fn from(value: [T; N]) -> Self {
        Self::_from(StackStorage::from(value))
    }
}

impl<T: Default + Copy, const N: usize> Default for ConcurrentStackRB<T, N> {
    fn default() -> Self {
        Self::from([T::default(); N])
    }
}
