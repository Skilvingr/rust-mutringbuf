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
            #[allow(refining_impl_trait)]
            impl<T> HeapSplit for $Struct<HeapStorage<T>> {
                type B = $Struct<HeapStorage<T>>;

                fn split<'buf>(
                    self,
                ) -> (
                    ProdIter<'buf, impl Deref<Target = Self::B> + RefDropManager>,
                    ConsIter<'buf, Self::B, impl Deref<Target = Self::B> + RefDropManager, false>,
                ) {
                    self.set_alive_iters(2);

                    let r = BufRef::<'buf, Self::B, true>::new(self);
                    (ProdIter::new(r.clone()), ConsIter::new(r))
                }

                fn split_mut<'buf>(
                    self,
                ) -> (
                    ProdIter<'buf, impl Deref<Target = Self::B> + RefDropManager>,
                    WorkIter<'buf, Self::B, impl Deref<Target = Self::B> + RefDropManager>,
                    ConsIter<'buf, Self::B, impl Deref<Target = Self::B> + RefDropManager, true>,
                ) {
                    self.set_alive_iters(3);

                    let r = BufRef::<'buf, Self::B, true>::new(self);
                    (
                        ProdIter::new(r.clone()),
                        WorkIter::new(r.clone()),
                        ConsIter::new(r),
                    )
                }
            }

            #[cfg(not(feature = "vmem"))]
            #[allow(refining_impl_trait)]
            impl<T, const N: usize> StackSplit for $Struct<StackStorage<T, N>> {
                type B = $Struct<StackStorage<T, N>>;

                fn split(
                    &'_ mut self,
                ) -> (
                    ProdIter<'_, impl Deref<Target = Self::B> + RefDropManager>,
                    ConsIter<'_, Self::B, impl Deref<Target = Self::B> + RefDropManager, false>,
                ) {
                    self.set_alive_iters(2);

                    let r = BufRef::<'_, Self::B, false>::from_ref(self);
                    (ProdIter::new(r.clone()), ConsIter::new(r))
                }

                fn split_mut(
                    &'_ mut self,
                ) -> (
                    ProdIter<'_, impl Deref<Target = Self::B> + RefDropManager>,
                    WorkIter<'_, Self::B, impl Deref<Target = Self::B> + RefDropManager>,
                    ConsIter<'_, Self::B, impl Deref<Target = Self::B> + RefDropManager, true>,
                ) {
                    self.set_alive_iters(3);

                    let r = BufRef::<'_, Self::B, false>::from_ref(self);
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
