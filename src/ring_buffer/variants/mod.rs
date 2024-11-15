use crate::{ConcurrentMutRingBuf, ConsIter, LocalMutRingBuf, MutRB, ProdIter, StackStorage, UnsafeSyncCell, WorkIter};

pub mod concurrent_rb;
pub mod local_rb;
pub mod ring_buffer_trait;
pub mod alloc_ext;


/// A stack-allocated ring buffer usable in concurrent environment.
pub type ConcurrentStackRB<T, const N: usize> = ConcurrentMutRingBuf<StackStorage<T, N>>;

impl<T, const N: usize> ConcurrentStackRB<T, N> {
    /// Creates a new concurrent stack-allocated buffer with given capacity and zeroed (uninitialised) elements.
    /// # Safety
    /// The buffer must be then initialised using proper [`ProdIter`] methods (`*_init` ones).
    pub unsafe fn new_zeroed() -> Self {
        let v: [UnsafeSyncCell<T>; N] = core::array::from_fn(|_| UnsafeSyncCell::new_zeroed());

        Self::_from(StackStorage::from(v))
    }
}


/// A stack-allocated ring buffer usable in local environment.
pub type LocalStackRB<T, const N: usize> = LocalMutRingBuf<StackStorage<T, N>>;

impl<T, const N: usize> LocalStackRB<T, N> {
    /// Creates a new local stack-allocated buffer with given capacity and zeroed (uninitialised) elements.
    /// # Safety
    /// The buffer must be then initialised using proper [`ProdIter`] methods (`*_init` ones).
    pub unsafe fn new_zeroed() -> Self {
        let v: [UnsafeSyncCell<T>; N] = core::array::from_fn(|_| UnsafeSyncCell::new_zeroed());

        Self::_from(StackStorage::from(v))
    }
}

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

pub(crate) mod impl_splits {
    macro_rules! impl_splits { ($Struct: tt) => {

        #[cfg(feature = "alloc")]
        impl<T> HeapSplit<$Struct<HeapStorage<T>>> for $Struct<HeapStorage<T>> {
            fn split<'buf>(self) -> (ProdIter<'buf, $Struct<HeapStorage<T>>>, ConsIter<'buf, $Struct<HeapStorage<T>>, false>) {
                self.set_prod_alive(true);
                self.set_cons_alive(true);

                let r = BufRef::new(self);
                (
                    ProdIter::new(r.clone()),
                    ConsIter::new(r),
                )
            }

            fn split_mut<'buf>(self) -> (ProdIter<'buf, $Struct<HeapStorage<T>>>, WorkIter<'buf, $Struct<HeapStorage<T>>>, ConsIter<'buf, $Struct<HeapStorage<T>>, true>) {
                self.set_prod_alive(true);
                self.set_work_alive(true);
                self.set_cons_alive(true);

                let r = BufRef::new(self);
                (
                    ProdIter::new(r.clone()),
                    WorkIter::new(r.clone()),
                    ConsIter::new(r),
                )
            }
        }

        impl<T, const N: usize> StackSplit<$Struct<StackStorage<T, N>>> for $Struct<StackStorage<T, N>> {
            fn split(&mut self) -> (ProdIter<$Struct<StackStorage<T, N>>>, ConsIter<$Struct<StackStorage<T, N>>, false>) {
                self.set_prod_alive(true);
                self.set_cons_alive(true);

                let r = BufRef::from_ref(self);
                (
                    ProdIter::new(r.clone()),
                    ConsIter::new(r),
                )
            }

            fn split_mut(&mut self) -> (ProdIter<$Struct<StackStorage<T, N>>>, WorkIter<$Struct<StackStorage<T, N>>>, ConsIter<$Struct<StackStorage<T, N>>, true>) {
                self.set_prod_alive(true);
                self.set_work_alive(true);
                self.set_cons_alive(true);

                let r = BufRef::from_ref(self);
                (
                    ProdIter::new(r.clone()),
                    WorkIter::new(r.clone()),
                    ConsIter::new(r),
                )
            }
        }
    }}

    pub(crate) use impl_splits;
}