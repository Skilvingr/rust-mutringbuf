use crate::UnsafeSyncCell;

pub mod heap;
pub mod stack;

pub(crate) trait MRBIndex<Idx: ?Sized> {
    type Output: ?Sized;

    fn _index(&self, index: Idx) -> &Self::Output;
}

/// Trait implemented by `*Storage` structs.
#[allow(clippy::len_without_is_empty)]
#[allow(private_bounds)]
pub trait Storage: MRBIndex<usize, Output = UnsafeSyncCell<Self::Item>> {
    type Item;

    /// Returns the underlying array as a const ptr.
    fn as_ptr(&self) -> *const UnsafeSyncCell<Self::Item>;
    /// Returns the underlying array as a mutable ptr.
    fn as_mut_ptr(&mut self) -> *mut UnsafeSyncCell<Self::Item>;
    /// Returns the length of the underlying array.
    fn len(&self) -> usize;
}

pub(crate) mod impl_splits {
    macro_rules! impl_splits {
        ($Struct: tt) => {
            #[cfg(feature = "alloc")]
            impl<T> HeapSplit<$Struct<HeapStorage<T>>> for $Struct<HeapStorage<T>> {
                fn split<'buf>(
                    self,
                ) -> (
                    ProdIter<'buf, $Struct<HeapStorage<T>>>,
                    ConsIter<'buf, $Struct<HeapStorage<T>>, false>,
                ) {
                    self.set_prod_alive(true);
                    self.set_cons_alive(true);

                    let r = BufRef::new(self);
                    (ProdIter::new(r.clone()), ConsIter::new(r))
                }

                fn split_mut<'buf>(
                    self,
                ) -> (
                    ProdIter<'buf, $Struct<HeapStorage<T>>>,
                    WorkIter<'buf, $Struct<HeapStorage<T>>>,
                    ConsIter<'buf, $Struct<HeapStorage<T>>, true>,
                ) {
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

            #[cfg(not(feature = "vmem"))]
            impl<T, const N: usize> StackSplit<$Struct<StackStorage<T, N>>>
                for $Struct<StackStorage<T, N>>
            {
                fn split(
                    &'_ mut self,
                ) -> (
                    ProdIter<'_, $Struct<StackStorage<T, N>>>,
                    ConsIter<'_, $Struct<StackStorage<T, N>>, false>,
                ) {
                    self.set_prod_alive(true);
                    self.set_cons_alive(true);

                    let r = BufRef::from_ref(self);
                    (ProdIter::new(r.clone()), ConsIter::new(r))
                }

                fn split_mut(
                    &'_ mut self,
                ) -> (
                    ProdIter<'_, $Struct<StackStorage<T, N>>>,
                    WorkIter<'_, $Struct<StackStorage<T, N>>>,
                    ConsIter<'_, $Struct<StackStorage<T, N>>, true>,
                ) {
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
        };
    }

    pub(crate) use impl_splits;
}
