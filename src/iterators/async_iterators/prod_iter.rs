use core::task::Waker;

use crate::iterators::ProdIter;
use crate::iterators::async_iterators::async_macros::{gen_common_futs_fn, waker_registerer};
use crate::iterators::async_iterators::{AsyncIterator, MRBFuture};
use crate::iterators::iterator_trait::MRBIterator;
use crate::iterators::iterator_trait::MutableSlice;
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, MutRB};

#[doc = r##"
Async version of [`ProdIter`].
"##]
pub struct AsyncProdIter<'buf, B: MutRB> {
    inner: ProdIter<'buf, B>,
    waker: Option<Waker>,
}
unsafe impl<B: ConcurrentRB + MutRB<Item = T>, T> Send for AsyncProdIter<'_, B> {}

impl<'buf, B: MutRB<Item = T>, T> AsyncIterator for AsyncProdIter<'buf, B> {
    type I = ProdIter<'buf, B>;
    type B = B;

    waker_registerer!();

    #[inline]
    fn inner(&self) -> &Self::I {
        &self.inner
    }
    #[inline]
    fn inner_mut(&mut self) -> &mut Self::I {
        &mut self.inner
    }
    fn into_sync(self) -> Self::I {
        self.inner
    }
    fn from_sync(iter: Self::I) -> Self {
        Self {
            inner: iter,
            waker: None,
        }
    }
}

impl<B: MutRB<Item = T>, T> AsyncProdIter<'_, B> {
    gen_common_futs_fn!();

    /// Async version of [`ProdIter::push`].
    pub fn push(&'_ mut self, item: T) -> MRBFuture<'_, Self, T, (), false> {
        #[inline]
        fn f<B: MutRB<Item = T>, T>(s: &mut AsyncProdIter<B>, item: T) -> Result<(), T> {
            s.inner_mut().push(item)
        }

        MRBFuture {
            iter: self,
            p: Some(item),
            f_r: None,
            f_m: Some(f),
        }
    }

    /// Async version of [`ProdIter::push_slice`].
    pub fn push_slice<'b>(&'_ mut self, slice: &'b [T]) -> MRBFuture<'_, Self, &'b [T], (), true>
    where
        T: Copy,
    {
        #[inline]
        fn f<B: MutRB<Item = T>, T: Copy>(
            s: &mut AsyncProdIter<B>,
            slice: &mut &[T],
        ) -> Option<()> {
            s.inner_mut().push_slice(slice)
        }

        MRBFuture {
            iter: self,
            p: Some(slice),
            f_r: Some(f),
            f_m: None,
        }
    }

    /// Async version of [`ProdIter::push_slice_clone`].
    pub fn push_slice_clone<'b>(
        &'_ mut self,
        slice: &'b [T],
    ) -> MRBFuture<'_, Self, &'b [T], (), true>
    where
        T: Clone,
    {
        #[inline]
        fn f<B: MutRB<Item = T>, T: Clone>(
            s: &mut AsyncProdIter<B>,
            slice: &mut &[T],
        ) -> Option<()> {
            s.inner_mut().push_slice_clone(slice)
        }

        MRBFuture {
            iter: self,
            p: Some(slice),
            f_r: Some(f),
            f_m: None,
        }
    }

    /// Async version of [`ProdIter::get_next_item_mut`].
    /// # Safety
    /// Same as [`ProdIter::get_next_item_mut`].
    pub unsafe fn get_next_item_mut<'b>(&'_ mut self) -> MRBFuture<'_, Self, (), &'b mut T, true> {
        #[inline]
        fn f<'b, B: MutRB<Item = T>, T>(s: &mut AsyncProdIter<B>, _: &mut ()) -> Option<&'b mut T> {
            unsafe { s.inner_mut().get_next_item_mut() }
        }

        MRBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None,
        }
    }

    /// Async version of [`ProdIter::get_next_item_mut_init`].
    pub fn get_next_item_mut_init(&'_ mut self) -> MRBFuture<'_, Self, (), *mut T, true> {
        #[inline]
        fn f<B: MutRB<Item = T>, T>(s: &mut AsyncProdIter<B>, _: &mut ()) -> Option<*mut T> {
            s.inner_mut().get_next_item_mut_init()
        }

        MRBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None,
        }
    }

    /// Async version of [`ProdIter::get_next_slices_mut`].
    /// # Safety
    /// See above.
    pub unsafe fn get_next_slices_mut<'b>(
        &'_ mut self,
        count: usize,
    ) -> MRBFuture<'_, Self, usize, MutableSlice<'b, T>, true> {
        #[inline]
        fn f<'b, B: MutRB<Item = T>, T>(
            s: &mut AsyncProdIter<B>,
            count: &mut usize,
        ) -> Option<MutableSlice<'b, T>> {
            unsafe { s.inner_mut().get_next_slices_mut(*count) }
        }

        MRBFuture {
            iter: self,
            p: Some(count),
            f_r: Some(f),
            f_m: None,
        }
    }
}
