#[allow(unused_imports)]
use crate::{DetachedWorkIter, MRBIterator, WorkIter};
use crate::AsyncDetachedWorkIter;
use crate::iterators::async_iterators::async_macros::{futures_import, gen_fut, waker_registerer};
use crate::iterators::sync_iterators::work_iter::WorkableSlice;
use crate::iterators::util_macros::delegate;
use crate::iterators::util_macros::muncher;
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, MutRB};

futures_import!();

#[doc = r##"
Async version of [`WorkIter`].
"##]

pub struct AsyncWorkIter<B: MutRB> {
    pub(crate) inner: WorkIter<B>,
    waker: Option<Waker>
}
unsafe impl<B: ConcurrentRB + MutRB<Item = T>, T> Send for AsyncWorkIter<B> {}


gen_fut!{
    GetWorkableFuture<'a, B: MutRB<Item = T>, T: 'a>,
    &'a mut AsyncWorkIter<B>,
    (),
    Option<&'a mut T>,
    self {
        let result = self.iter.inner.get_workable();

        if let Some(res) = result {
            break Poll::Ready(Some(res));
        }
    }
}

gen_fut!{
    GetWorkableSliceExactFuture<'a, B: MutRB<Item = T>, T: 'a>,
    &'a mut AsyncWorkIter<B>,
    usize,
    Option<WorkableSlice<'a, T>>,
    self {
        let count = self._item;
        let result = self.iter.inner.get_workable_slice_exact(count);

        if let Some(res) = result {
            break Poll::Ready(Some(res));
        }
    }
}

gen_fut!{
    GetWorkableSliceAvailFuture<'a, B: MutRB<Item = T>, T: 'a>,
    &'a mut AsyncWorkIter<B>,
    (),
    Option<WorkableSlice<'a, T>>,
    self {
        let result = self.iter.inner.get_workable_slice_avail();

        if let Some(res) = result {
            break Poll::Ready(Some(res));
        }
    }
}

gen_fut!{
    GetWorkableSliceMultipleOfFuture<'a, B: MutRB<Item = T>, T: 'a>,
    &'a mut AsyncWorkIter<B>,
    usize,
    Option<WorkableSlice<'a, T>>,
    self {
        let count = self._item;
        let result = self.iter.inner.get_workable_slice_multiple_of(count);

        if let Some(res) = result {
            break Poll::Ready(Some(res));
        }
    }
}

impl<B: MutRB<Item = T>, T> AsyncWorkIter<B> {
    pub fn from_sync(iter: WorkIter<B>) -> Self {
        Self {
            inner: iter,
            waker: None,
        }
    }

    pub fn into_sync(self) -> WorkIter<B> {
        self.inner
    }

    waker_registerer!();
    delegate!(WorkIter, pub fn is_prod_alive(&self) -> bool);
    delegate!(WorkIter, pub fn is_cons_alive(&self) -> bool);
    delegate!(WorkIter, pub fn prod_index(&self) -> usize);
    delegate!(WorkIter, pub fn cons_index(&self) -> usize);
    delegate!(WorkIter, pub fn index(&self) -> usize);
    delegate!(WorkIter, pub unsafe fn advance(&(mut) self, count: usize));
    delegate!(WorkIter, pub fn available(&(mut) self) -> usize);
    delegate!(WorkIter, pub fn reset_index(&(mut) self));


    /// Detaches the iterator yielding a [`DetachedWorkIter`].
    #[inline]
    pub fn detach(self) -> AsyncDetachedWorkIter<B> {
        AsyncDetachedWorkIter::from_work(self)
    }

    /// Returns a mutable references to the current value.
    ///
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn get_workable(&mut self) -> GetWorkableFuture<'_, B, T> {
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
    pub fn get_workable_slice_exact(&mut self, count: usize) -> GetWorkableSliceExactFuture<'_, B, T> {
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
    pub fn get_workable_slice_avail(&mut self) -> GetWorkableSliceAvailFuture<'_, B, T> {
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
    pub fn get_workable_slice_multiple_of(&mut self, rhs: usize) -> GetWorkableSliceMultipleOfFuture<'_, B, T> {
        GetWorkableSliceMultipleOfFuture {
            iter: self,
            _item: rhs,
        }
    }
}