use core::cell::{Cell, UnsafeCell};
use core::marker::PhantomData;

use crate::{ConsIter, LocalStackRB, ProdIter, WorkIter};
use crate::ring_buffer::storage::stack::StackStorage;
use crate::ring_buffer::storage::storage_trait::Storage;
use crate::ring_buffer::variants::ring_buffer_trait::{IterManager, LocalRB, MutRB, StorageManager};
use crate::ring_buffer::wrappers::buf_ref::BufRef;

pub struct LocalMutRingBuf<S: Storage<T>, T> {
    pub(crate) inner_len: usize,
    pub(crate) inner: UnsafeCell<S>,

    pub(crate) prod_idx: Cell<usize>,
    pub(crate) work_idx: Cell<usize>,
    pub(crate) cons_idx: Cell<usize>,

    pub(crate) prod_alive: Cell<bool>,
    pub(crate) work_alive: Cell<bool>,
    pub(crate) cons_alive: Cell<bool>,

    _phantom: PhantomData<T>
}

impl<S: Storage<T>, T> MutRB<T> for LocalMutRingBuf<S, T> {}

impl<S: Storage<T>, T> LocalRB for LocalMutRingBuf<S, T> {}

impl<S: Storage<T>, T> LocalMutRingBuf<S, T> {
    /// Consumes the buffer, yielding three iterators. See:
    /// - [`ProdIter`];
    /// - [`WorkIter`];
    /// - [`ConsIter`].
    pub fn split_mut(self) -> (ProdIter<LocalMutRingBuf<S, T>, T>, WorkIter<LocalMutRingBuf<S, T>, T>, ConsIter<LocalMutRingBuf<S, T>, T, true>) {
        self.prod_alive.set(true);
        self.work_alive.set(true);
        self.cons_alive.set(true);

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
    pub fn split(self) -> (ProdIter<LocalMutRingBuf<S, T>, T>, ConsIter<LocalMutRingBuf<S, T>, T, false>) {
        self.prod_alive.set(true);
        self.cons_alive.set(true);

        let r = BufRef::new(self);
        (
            ProdIter::new(r.clone()),
            ConsIter::new(r),
        )
    }

    pub(crate) fn _from(value: S) -> LocalMutRingBuf<S, T> {
        assert!(value.len() > 1);

        LocalMutRingBuf {
            inner_len: value.len(),
            inner: value.into(),

            prod_idx: 0.into(),
            work_idx: 0.into(),
            cons_idx: 0.into(),

            prod_alive: false.into(),
            work_alive: false.into(),
            cons_alive: false.into(),
            _phantom: PhantomData
        }
    }
}

impl<S: Storage<T>, T> IterManager for LocalMutRingBuf<S, T> {
    #[inline(always)]
    fn prod_index(&self) -> usize {
        self.prod_idx.get()
    }

    #[inline(always)]
    fn work_index(&self) -> usize {
        self.work_idx.get()
    }

    #[inline(always)]
    fn cons_index(&self) -> usize {
        self.cons_idx.get()
    }

    #[inline(always)]
    fn set_prod_index(&self, index: usize) {
        self.prod_idx.set(index);
    }

    #[inline(always)]
    fn set_work_index(&self, index: usize) {
        self.work_idx.set(index);
    }

    #[inline(always)]
    fn set_cons_index(&self, index: usize) {
        self.cons_idx.set(index);
    }

    fn prod_alive(&self) -> bool {
        self.prod_alive.get()
    }

    fn work_alive(&self) -> bool {
        self.work_alive.get()
    }

    fn cons_alive(&self) -> bool {
        self.cons_alive.get()
    }

    fn set_prod_alive(&self, alive: bool) {
        self.prod_alive.set(alive);
    }

    fn set_work_alive(&self, alive: bool) {
        self.work_alive.set(alive);
    }

    fn set_cons_alive(&self, alive: bool) {
        self.cons_alive.set(alive);
    }
}

impl<S: Storage<T>, T> StorageManager for LocalMutRingBuf<S, T> {
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

impl<T, const N: usize> From<[T; N]> for LocalStackRB<T, N> {
    fn from(value: [T; N]) -> Self {
        Self::_from(StackStorage::from(value))
    }
}

impl<T: Default + Copy, const N: usize> Default for LocalStackRB<T, N> {
    fn default() -> Self {
        Self::from([T::default(); N])
    }
}
