use core::marker::PhantomData;
use core::task::Waker;

use crate::Storage;
use crate::iterators::ConsIter;
use crate::iterators::async_iterators::async_macros::gen_common_futs_fn;
use crate::iterators::async_iterators::{AsyncIterator, MRBFuture};
use crate::iterators::iterator_trait::MutableSlice;
use crate::iterators::iterator_trait::{MRBIterator, NonMutableSlice};
use crate::iterators::util_macros::delegate;
use crate::ring_buffer::variants::async_rb::AsyncMutRingBuf;
use crate::ring_buffer::wrappers::buf_ref::BufRef;

#[doc = r##"
Async version of [`ConsIter`].
"##]
pub struct AsyncConsIter<'buf, S: Storage, const W: bool> {
    inner: ConsIter<'buf, AsyncMutRingBuf<S>, W>,
    waker: BufRef<'buf, AsyncMutRingBuf<S>>,
}
unsafe impl<S: Storage, const W: bool> Send for AsyncConsIter<'_, S, W> {}

impl<'buf, S: Storage, const W: bool> AsyncIterator<'buf> for AsyncConsIter<'buf, S, W> {
    type I = ConsIter<'buf, AsyncMutRingBuf<S>, W>;
    type S = S;

    fn register_waker(&self, waker: &Waker) {
        (*self.waker).cons_waker.register(waker);
    }

    fn take_waker(&self) -> Option<Waker> {
        (*self.waker).cons_waker.take()
    }

    fn wake_next(&self) {
        (*self.waker).prod_waker.wake()
    }

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
    fn from_sync(iter: Self::I, waker: BufRef<'buf, AsyncMutRingBuf<S>>) -> Self {
        Self { inner: iter, waker }
    }
}

impl<'buf, S: Storage<Item = T>, T, const W: bool> AsyncConsIter<'buf, S, W> {
    gen_common_futs_fn!( 'buf );

    delegate!(ConsIter, pub fn reset_index(&(mut) self));

    /// Async version of [`ConsIter::peek_ref`].
    pub fn peek_ref<'b>(&'b mut self) -> MRBFuture<'buf, 'b, Self, (), &'b T, true> {
        #[inline]
        fn f<'b, S: Storage<Item = T>, const W: bool, T>(
            s: &mut AsyncConsIter<S, W>,
            _: &mut (),
        ) -> Option<&'b T> {
            s.inner_mut().peek_ref()
        }

        MRBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ConsIter::peek_slice`].
    pub fn peek_slice<'b>(
        &'b mut self,
        count: usize,
    ) -> MRBFuture<'buf, 'b, Self, usize, NonMutableSlice<'b, T>, true> {
        #[inline]
        fn f<'b, S: Storage<Item = T>, const W: bool, T>(
            s: &mut AsyncConsIter<S, W>,
            count: &mut usize,
        ) -> Option<NonMutableSlice<'b, T>> {
            s.inner_mut().peek_slice(*count)
        }

        MRBFuture {
            iter: self,
            p: Some(count),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ConsIter::peek_available`].
    pub fn peek_available<'b>(
        &'b mut self,
    ) -> MRBFuture<'buf, 'b, Self, (), NonMutableSlice<'b, T>, true> {
        #[inline]
        fn f<'b, S: Storage<Item = T>, const W: bool, T>(
            s: &mut AsyncConsIter<S, W>,
            _: &mut (),
        ) -> Option<NonMutableSlice<'b, T>> {
            s.inner_mut().peek_available()
        }

        MRBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ConsIter::pop`].
    /// # Safety
    /// See above.
    pub fn pop<'b>(&'b mut self) -> MRBFuture<'buf, 'b, Self, (), T, true> {
        #[inline]
        fn f<S: Storage<Item = T>, const W: bool, T>(
            s: &mut AsyncConsIter<S, W>,
            _: &mut (),
        ) -> Option<T> {
            s.inner_mut().pop()
        }

        MRBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ConsIter::pop_move`].
    /// # Safety
    /// See above.
    pub unsafe fn pop_move<'b>(&'b mut self) -> MRBFuture<'buf, 'b, Self, (), T, true> {
        #[inline]
        fn f<S: Storage<Item = T>, const W: bool, T>(
            s: &mut AsyncConsIter<S, W>,
            _: &mut (),
        ) -> Option<T> {
            unsafe { s.inner_mut().pop_move() }
        }

        MRBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ConsIter::copy_item`].
    pub fn copy_item<'b>(
        &'b mut self,
        dst: &'b mut T,
    ) -> MRBFuture<'buf, 'b, Self, &'b mut T, (), true>
    where
        T: Copy,
    {
        #[inline]
        fn f<S: Storage<Item = T>, const W: bool, T: Copy>(
            s: &mut AsyncConsIter<S, W>,
            dst: &mut &mut T,
        ) -> Option<()> {
            s.inner_mut().copy_item(*dst)
        }

        MRBFuture {
            iter: self,
            p: Some(dst),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ConsIter::clone_item`].
    pub fn clone_item<'b>(
        &'b mut self,
        dst: &'b mut T,
    ) -> MRBFuture<'buf, 'b, Self, &'b mut T, (), true>
    where
        T: Clone,
    {
        #[inline]
        fn f<S: Storage<Item = T>, const W: bool, T: Clone>(
            s: &mut AsyncConsIter<S, W>,
            dst: &mut &mut T,
        ) -> Option<()> {
            s.inner_mut().clone_item(*dst)
        }

        MRBFuture {
            iter: self,
            p: Some(dst),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ConsIter::copy_slice`].
    pub fn copy_slice<'b>(
        &'b mut self,
        dst: &'b mut [T],
    ) -> MRBFuture<'buf, 'b, Self, &'b mut [T], (), true>
    where
        T: Copy,
    {
        #[inline]
        fn f<S: Storage<Item = T>, const W: bool, T: Copy>(
            s: &mut AsyncConsIter<S, W>,
            dst: &mut &mut [T],
        ) -> Option<()> {
            s.inner_mut().copy_slice(dst)
        }

        MRBFuture {
            iter: self,
            p: Some(dst),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ConsIter::clone_slice`].
    pub fn clone_slice<'b>(
        &'b mut self,
        dst: &'b mut [T],
    ) -> MRBFuture<'buf, 'b, Self, &'b mut [T], (), true>
    where
        T: Clone,
    {
        #[inline]
        fn f<S: Storage<Item = T>, const W: bool, T: Clone>(
            s: &mut AsyncConsIter<S, W>,
            dst: &mut &mut [T],
        ) -> Option<()> {
            s.inner_mut().clone_slice(dst)
        }

        MRBFuture {
            iter: self,
            p: Some(dst),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }
}
