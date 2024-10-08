#[allow(unused_imports)]
use crate::{MRBIterator, WorkIter};
use crate::iterators::async_iterators::async_macros::{futures_import, gen_common_futs, gen_fut, waker_registerer};
use crate::iterators::async_iterators::AsyncIterator;
use crate::iterators::util_macros::delegate;
use crate::iterators::util_macros::muncher;
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, MutRB};

futures_import!();

#[doc = r##"
Async version of [`WorkIter`].
"##]

pub struct AsyncWorkIter<'buf, B: MutRB> {
    pub(crate) inner: WorkIter<'buf, B>,
    waker: Option<Waker>
}
unsafe impl<'buf, B: ConcurrentRB + MutRB<Item = T>, T> Send for AsyncWorkIter<'buf, B> {}

impl<'buf, B: MutRB<Item = T>, T> AsyncIterator for AsyncWorkIter<'buf, B> {
    type I = WorkIter<'buf, B>;

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

gen_common_futs! (&'a mut AsyncWorkIter<'buf, B>);

impl<'buf, B: MutRB<Item = T>, T> AsyncWorkIter<'buf, B> {
    waker_registerer!();
    delegate!(WorkIter, pub fn is_prod_alive(&self) -> bool);
    delegate!(WorkIter, pub fn is_cons_alive(&self) -> bool);
    delegate!(WorkIter, pub fn prod_index(&self) -> usize);
    delegate!(WorkIter, pub fn cons_index(&self) -> usize);
    delegate!(WorkIter, pub fn index(&self) -> usize);
    delegate!(WorkIter, pub unsafe fn advance(&(mut) self, count: usize));
    delegate!(WorkIter, pub fn available(&(mut) self) -> usize);
    delegate!(WorkIter, pub fn reset_index(&(mut) self));

    /// Returns a mutable references to the current value.
    ///
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn get_workable(&mut self) -> GetWorkableFuture<'buf, '_, B, T> {
        GetWorkableFuture {
            iter: self,
            _item: (),
        }
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to `count`.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn get_workable_slice_exact(&mut self, count: usize) -> GetWorkableSliceExactFuture<'buf, '_, B, T> {
        GetWorkableSliceExactFuture {
            iter: self,
            _item: count,
        }
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to [`Self::available()`].
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn get_workable_slice_avail(&mut self) -> GetWorkableSliceAvailFuture<'buf, '_, B, T> {
        GetWorkableSliceAvailFuture {
            iter: self,
            _item: (),
        }
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to the
    /// maximum multiple of `modulo`.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn get_workable_slice_multiple_of(&mut self, rhs: usize) -> GetWorkableSliceMultipleOfFuture<'buf, '_, B, T> {
        GetWorkableSliceMultipleOfFuture {
            iter: self,
            _item: rhs,
        }
    }
}