use core::cell::UnsafeCell;
use core::num::NonZeroUsize;

use crate::{ConsIter, LocalStackRB, ProdIter, WorkIter};
#[cfg(feature = "alloc")]
use crate::HeapSplit;
#[cfg(feature = "alloc")]
use crate::HeapStorage;
use crate::ring_buffer::storage::stack::StackStorage;
use crate::ring_buffer::storage::storage_trait::Storage;
use crate::ring_buffer::variants::impl_splits::impl_splits;
use crate::ring_buffer::variants::ring_buffer_trait::{IterManager, MutRB, StorageManager};
use crate::ring_buffer::wrappers::buf_ref::BufRef;
use crate::StackSplit;

pub struct LocalMutRingBuf<S: Storage> {
    pub(crate) inner_len: NonZeroUsize,
    pub(crate) inner: UnsafeCell<S>,

    pub(crate) prod_idx: UnsafeCell<usize>,
    pub(crate) work_idx: UnsafeCell<usize>,
    pub(crate) cons_idx: UnsafeCell<usize>,

    pub(crate) prod_alive: UnsafeCell<bool>,
    pub(crate) work_alive: UnsafeCell<bool>,
    pub(crate) cons_alive: UnsafeCell<bool>,
}

impl<S: Storage<Item = T>, T> MutRB for LocalMutRingBuf<S> {
    type Item = T;
}

impl_splits!(LocalMutRingBuf);

impl<S: Storage<Item = T>, T> LocalMutRingBuf<S> {
    pub(crate) fn _from(value: S) -> LocalMutRingBuf<S> {
        LocalMutRingBuf {
            inner_len: NonZeroUsize::new(value.len()).unwrap(),
            inner: value.into(),

            prod_idx: 0.into(),
            work_idx: 0.into(),
            cons_idx: 0.into(),

            prod_alive: false.into(),
            work_alive: false.into(),
            cons_alive: false.into()
        }
    }
}

impl<S: Storage> IterManager for LocalMutRingBuf<S> {
    #[inline(always)]
    fn prod_index(&self) -> usize {
        unsafe { *self.prod_idx.get() }
    }

    #[inline(always)]
    fn work_index(&self) -> usize {
        unsafe { *self.work_idx.get() }
    }

    #[inline(always)]
    fn cons_index(&self) -> usize {
        unsafe { *self.cons_idx.get() }
    }

    #[inline(always)]
    fn set_prod_index(&self, index: usize) {
        unsafe { *self.prod_idx.get() = index; }
    }

    #[inline(always)]
    fn set_work_index(&self, index: usize) {
        unsafe { *self.work_idx.get() = index; }
    }

    #[inline(always)]
    fn set_cons_index(&self, index: usize) {
        unsafe { *self.cons_idx.get() = index; }
    }

    fn prod_alive(&self) -> bool {
        unsafe { *self.prod_alive.get() }
    }

    fn work_alive(&self) -> bool {
        unsafe { *self.work_alive.get() }
    }

    fn cons_alive(&self) -> bool {
        unsafe { *self.cons_alive.get() }
    }

    fn set_prod_alive(&self, alive: bool) {
        unsafe { *self.prod_alive.get() = alive; }
    }

    fn set_work_alive(&self, alive: bool) {
        unsafe { *self.work_alive.get() = alive; }
    }

    fn set_cons_alive(&self, alive: bool) {
        unsafe { *self.cons_alive.get() = alive; }
    }
}

impl<S: Storage<Item = T>, T> StorageManager for LocalMutRingBuf<S> {
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
