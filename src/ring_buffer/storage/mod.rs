use core::ops::Index;

use crate::UnsafeSyncCell;

pub mod heap;
pub mod stack;

/// Trait implemented by `*Storage` structs.
#[allow(clippy::len_without_is_empty)]
pub trait Storage: Index<usize, Output = UnsafeSyncCell<Self::Item>>
{
    type Item;

    /// Returns the underlying array as a const ptr.
    fn as_ptr(&self) -> *const Self::Output;
    /// Returns the underlying array as a mutable ptr.
    fn as_mut_ptr(&mut self) -> *mut Self::Output;
    /// Returns the length of the underlying array.
    fn len(&self) -> usize;
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

        #[cfg(not(feature = "vmem"))]
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