use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicUsize};
use core::sync::atomic::Ordering::{Acquire, Relaxed, Release};

use crossbeam_utils::CachePadded;

use crate::{ConcurrentStackRB, ConsIter, ProdIter, WorkIter};
use crate::ring_buffer::storage::stack::StackStorage;
use crate::ring_buffer::storage::storage_trait::Storage;
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, IterManager, MutRB, StorageManager};
use crate::ring_buffer::wrappers::buf_ref::BufRef;

pub struct ConcurrentMutRingBuf<S: Storage<T>, T> {
    inner_len: usize,
    inner: UnsafeCell<S>,

    prod_idx: CachePadded<AtomicUsize>,
    work_idx: CachePadded<AtomicUsize>,
    cons_idx: CachePadded<AtomicUsize>,

    prod_alive: AtomicBool,
    work_alive: AtomicBool,
    cons_alive: AtomicBool,

    _phantom: PhantomData<T>
}

impl<S: Storage<T>, T> MutRB<T> for ConcurrentMutRingBuf<S, T> {}

impl<S: Storage<T>, T> ConcurrentRB for ConcurrentMutRingBuf<S, T> {}

impl<S: Storage<T>, T> ConcurrentMutRingBuf<S, T> {
    /// Consumes the buffer, yielding three iterators. See:
    /// - [`ProdIter`];
    /// - [`WorkIter`];
    /// - [`ConsIter`].
    pub fn split_mut(self) -> (ProdIter<ConcurrentMutRingBuf<S, T>, T>, WorkIter<ConcurrentMutRingBuf<S, T>, T>, ConsIter<ConcurrentMutRingBuf<S, T>, T, true>) {
        self.prod_alive.store(true, Relaxed);
        self.work_alive.store(true, Relaxed);
        self.cons_alive.store(true, Relaxed);

        let r = BufRef::new(self);
        (
            ProdIter::new(r.clone()),
            WorkIter::new(r.clone()),
            ConsIter::new(r),
        )
    }

    /// Consumes the buffer, yielding two iterators. See:
    /// - [`ProdIter`];
    /// - [`ConsIter`].
    pub fn split(self) -> (ProdIter<ConcurrentMutRingBuf<S, T>, T>, ConsIter<ConcurrentMutRingBuf<S, T>, T, false>) {
        self.prod_alive.store(true, Relaxed);
        self.cons_alive.store(true, Relaxed);

        let r = BufRef::new(self);
        (
            ProdIter::new(r.clone()),
            ConsIter::new(r),
        )
    }

    pub(crate) fn _from(value: S) -> ConcurrentMutRingBuf<S, T> {
        assert!(value.len() > 1);

        ConcurrentMutRingBuf {
            inner_len: value.len(),
            inner: value.into(),

            prod_idx: CachePadded::new(0.into()),
            work_idx: CachePadded::new(0.into()),
            cons_idx: CachePadded::new(0.into()),

            prod_alive: AtomicBool::default(),
            work_alive: AtomicBool::default(),
            cons_alive: AtomicBool::default(),
            _phantom: PhantomData
        }
    }
}

impl<S: Storage<T>, T> IterManager for ConcurrentMutRingBuf<S, T> {
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

impl<S: Storage<T>, T> StorageManager for ConcurrentMutRingBuf<S, T> {
    type StoredType = T;
    type S = S;

    #[inline]
    fn inner(&self) -> &S {
        unsafe { &(*self.inner.get()) }
    }

    #[inline]
    fn inner_mut(&mut self) -> &mut S {
        unsafe { &mut (*self.inner.get()) }
    }

    #[inline]
    fn inner_len(&self) -> usize {
        self.inner_len
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
