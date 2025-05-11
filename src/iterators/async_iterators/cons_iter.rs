use core::task::Waker;

use crate::iterators::async_iterators::async_macros::{gen_common_futs_fn, waker_registerer};
use crate::iterators::async_iterators::{AsyncIterator, MRBFuture};
use crate::iterators::iterator_trait::{MRBIterator, NonMutableSlice};
use crate::iterators::iterator_trait::MutableSlice;
use crate::iterators::util_macros::delegate;
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, MutRB};
use crate::iterators::ConsIter;

#[doc = r##"
Async version of [`ConsIter`].
"##]
pub struct AsyncConsIter<'buf, B: MutRB, const W: bool> {
    inner: ConsIter<'buf, B, W>,
    waker: Option<Waker>
}
unsafe impl<B: ConcurrentRB + MutRB<Item = T>, T, const W: bool> Send for AsyncConsIter<'_, B, W> {}

impl<'buf, B: MutRB<Item = T>, T, const W: bool> AsyncIterator for AsyncConsIter<'buf, B, W> {
    type I = ConsIter<'buf, B, W>;
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

impl<B: MutRB<Item = T>, T, const W: bool> AsyncConsIter<'_, B, W> {
    gen_common_futs_fn!();

    delegate!(ConsIter, pub fn reset_index(&(mut) self));

    /// Async version of [`ConsIter::peek_ref`].
    pub fn peek_ref<'b>(&mut self) -> MRBFuture<Self, (), &'b T, true> {
        #[inline]
        fn f<'b, B: MutRB<Item = T>, const W: bool, T>(s: &mut AsyncConsIter<B, W>, _: &mut ()) -> Option<&'b T> {
            s.inner_mut().peek_ref()
        }

        MRBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None
        }
    }

    /// Async version of [`ConsIter::peek_slice`].
    pub fn peek_slice<'b>(&mut self, count: usize) -> MRBFuture<Self, usize, NonMutableSlice<'b, T>, true> {
        #[inline]
        fn f<'b, B: MutRB<Item = T>, const W: bool, T>(s: &mut AsyncConsIter<B, W>, count: &mut usize) -> Option<NonMutableSlice<'b, T>> {
            s.inner_mut().peek_slice(*count)
        }

        MRBFuture {
            iter: self,
            p: Some(count),
            f_r: Some(f),
            f_m: None
        }
    }

    /// Async version of [`ConsIter::peek_available`].
    pub fn peek_available<'b>(&mut self) -> MRBFuture<Self, (), NonMutableSlice<'b, T>, true> {
        #[inline]
        fn f<'b, B: MutRB<Item = T>, const W: bool, T>(s: &mut AsyncConsIter<B, W>, _: &mut ()) -> Option<NonMutableSlice<'b, T>> {
            s.inner_mut().peek_available()
        }

        MRBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None
        }
    }

    /// Async version of [`ConsIter::pop`].
    /// # Safety
    /// See above.
    pub fn pop(&mut self) -> MRBFuture<Self, (), T, true> {
        #[inline]
        fn f<B: MutRB<Item = T>, const W: bool, T>(s: &mut AsyncConsIter<B, W>, _: &mut ()) -> Option<T> {
            s.inner_mut().pop()
        }

        MRBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None
        }
    }

    /// Async version of [`ConsIter::pop_move`].
    /// # Safety
    /// See above.
    pub unsafe fn pop_move(&mut self) -> MRBFuture<Self, (), T, true> {
        #[inline]
        fn f<B: MutRB<Item = T>, const W: bool, T>(s: &mut AsyncConsIter<B, W>, _: &mut ()) -> Option<T> {
            unsafe { s.inner_mut().pop_move() }
        }

        MRBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None
        }
    }

    /// Async version of [`ConsIter::copy_item`].
    pub fn copy_item<'b>(&mut self, dst: &'b mut T) -> MRBFuture<Self, &'b mut T, (), true>
    where T: Copy {
        #[inline]
        fn f<B: MutRB<Item = T>, const W: bool, T: Copy>(s: &mut AsyncConsIter<B, W>, dst: &mut&mut T) -> Option<()> {
            s.inner_mut().copy_item(*dst)
        }

        MRBFuture {
            iter: self,
            p: Some(dst),
            f_r: Some(f),
            f_m: None
        }
    }

    /// Async version of [`ConsIter::clone_item`].
    pub fn clone_item<'b>(&mut self, dst: &'b mut T) -> MRBFuture<Self, &'b mut T, (), true>
    where T: Clone {
        #[inline]
        fn f<B: MutRB<Item = T>, const W: bool, T: Clone>(s: &mut AsyncConsIter<B, W>, dst: &mut&mut T) -> Option<()> {
            s.inner_mut().clone_item(*dst)
        }

        MRBFuture {
            iter: self,
            p: Some(dst),
            f_r: Some(f),
            f_m: None
        }
    }

    /// Async version of [`ConsIter::copy_slice`].
    pub fn copy_slice<'b>(&mut self, dst: &'b mut [T]) -> MRBFuture<Self, &'b mut [T], (), true>
    where T: Copy {
        #[inline]
        fn f<B: MutRB<Item = T>, const W: bool, T: Copy>(s: &mut AsyncConsIter<B, W>, dst: &mut&mut [T]) -> Option<()> {
            s.inner_mut().copy_slice(dst)
        }

        MRBFuture {
            iter: self,
            p: Some(dst),
            f_r: Some(f),
            f_m: None
        }
    }

    /// Async version of [`ConsIter::clone_slice`].
    pub fn clone_slice<'b>(&mut self, dst: &'b mut [T]) -> MRBFuture<Self, &'b mut [T], (), true>
    where T: Clone {
        #[inline]
        fn f<B: MutRB<Item = T>, const W: bool, T: Clone>(s: &mut AsyncConsIter<B, W>, dst: &mut&mut [T]) -> Option<()> {
            s.inner_mut().clone_slice(dst)
        }

        MRBFuture {
            iter: self,
            p: Some(dst),
            f_r: Some(f),
            f_m: None
        }
    }
}
