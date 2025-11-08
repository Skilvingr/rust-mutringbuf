#![cfg_attr(doc, doc(cfg(feature = "async")))]
#![cfg(any(feature = "async", doc))]

//! Async iterators.

use crate::MRBIterator;
use crate::Storage;
use crate::iterators::async_iterators::detached::AsyncDetached;
use crate::iterators::util_macros::delegate;
use crate::iterators::util_macros::muncher;
use crate::ring_buffer::variants::async_rb::AsyncMutRingBuf;
use crate::ring_buffer::wrappers::buf_ref::BufRef;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

pub(crate) mod cons_iter;
pub(crate) mod detached;
pub(crate) mod prod_iter;
pub(crate) mod work_iter;

/// Trait implemented by async iterators.
pub trait AsyncIterator<'buf> {
    type I: MRBIterator;
    type S: Storage;

    fn register_waker(&self, waker: &Waker);
    fn take_waker(&self) -> Option<Waker>;
    fn wake_next(&self);

    fn inner(&self) -> &Self::I;
    fn inner_mut(&mut self) -> &mut Self::I;

    fn into_sync(self) -> Self::I;

    fn from_sync(iter: Self::I, waker: BufRef<'buf, AsyncMutRingBuf<Self::S>>) -> Self;

    fn detach(self) -> AsyncDetached<'buf, Self, AsyncMutRingBuf<Self::S>>
    where
        Self: Sized,
    {
        AsyncDetached::from_iter(self)
    }

    delegate!(MRBIterator, fn is_prod_alive(&self) -> bool);
    delegate!(MRBIterator, fn is_work_alive(&self) -> bool);
    delegate!(MRBIterator, fn is_cons_alive(&self) -> bool);
    delegate!(MRBIterator, fn prod_index(&self) -> usize);
    delegate!(MRBIterator, fn work_index(&self) -> usize);
    delegate!(MRBIterator, fn cons_index(&self) -> usize);
    delegate!(MRBIterator, fn index(&self) -> usize);
    delegate!(MRBIterator, fn available(&(mut) self) -> usize);
}

/// Future returned by methods in async iterators.
pub struct MRBFuture<'buf, 'a, I, P, O, const R: bool>
where
    I: AsyncIterator<'buf>,
{
    iter: &'a mut I,
    p: Option<P>,
    f_r: Option<fn(&mut I, &mut P) -> Option<O>>,
    f_m: Option<fn(&mut I, P) -> Result<O, P>>,
    phantom: PhantomData<&'buf ()>,
}

impl<'buf, 'a, I: AsyncIterator<'buf>, P, O, const R: bool> Unpin
    for MRBFuture<'buf, 'a, I, P, O, R>
{
}

impl<'buf, 'a, I: AsyncIterator<'buf>, P, O, const R: bool> Future
    for MRBFuture<'buf, 'a, I, P, O, R>
{
    type Output = Option<O>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let f_r = self.f_r.take();
        let f_m = self.f_m.take();
        let p = self.p.take();

        let res = if R {
            let mut p = p.unwrap();
            let ret = f_r.as_ref().unwrap()(self.iter, &mut p);
            ret.ok_or(p)
        } else {
            f_m.as_ref().unwrap()(self.iter, p.unwrap())
        };

        match res {
            Ok(r) => {
                return Poll::Ready(Some(r));
            }
            Err(p) => {
                self.f_r = f_r;
                self.f_m = f_m;
                self.p = Some(p);
            }
        }

        self.iter.register_waker(cx.waker());
        Poll::Pending
    }
}

pub(crate) mod async_macros {
    macro_rules! gen_common_futs_fn {
        ($LT: lifetime) => {
            /// Async version of [`MRBIterator::get_workable`].
            pub fn get_workable<'b>(&'b mut self) -> MRBFuture<$LT, 'b, Self, (), &'b mut T, true> {
                fn f<'buf, 'b, II: MRBIterator<Item = T>, I: AsyncIterator<'buf, I = II>, T>(
                    s: &mut I,
                    _: &mut (),
                ) -> Option<&'b mut T> {
                    s.inner_mut().get_workable()
                }

                MRBFuture {
                    iter: self,
                    p: Some(()),
                    f_r: Some(f),
                    f_m: None,
                    phantom: PhantomData,
                }
            }

            /// Async version of [`MRBIterator::get_workable_slice_exact`].
            pub fn get_workable_slice_exact<'b>(
                &'b mut self,
                count: usize,
            ) -> MRBFuture<$LT, 'b, Self, usize, MutableSlice<'b, T>, true> {
                fn f<'buf, 'b, II: MRBIterator<Item = T>, I: AsyncIterator<'buf, I = II>, T>(
                    s: &mut I,
                    count: &mut usize,
                ) -> Option<MutableSlice<'b, T>> {
                    s.inner_mut().get_workable_slice_exact(*count)
                }

                MRBFuture {
                    iter: self,
                    p: Some(count),
                    f_r: Some(f),
                    f_m: None,
                    phantom: PhantomData,
                }
            }

            /// Async version of [`MRBIterator::get_workable_slice_avail`].
            pub fn get_workable_slice_avail<'b>(
                &'b mut self,
            ) -> MRBFuture<$LT, 'b, Self, (), MutableSlice<'b, T>, true> {
                fn f<'buf, 'b, II: MRBIterator<Item = T>, I: AsyncIterator<'buf, I = II>, T>(
                    s: &mut I,
                    _: &mut (),
                ) -> Option<MutableSlice<'b, T>> {
                    s.inner_mut().get_workable_slice_avail()
                }

                MRBFuture {
                    iter: self,
                    p: Some(()),
                    f_r: Some(f),
                    f_m: None,
                    phantom: PhantomData,
                }
            }

            /// Async version of [`MRBIterator::get_workable_slice_multiple_of`].
            pub fn get_workable_slice_multiple_of<'b>(
                &'b mut self,
                count: usize,
            ) -> MRBFuture<$LT, 'b, Self, usize, MutableSlice<'b, T>, true> {
                fn f<'buf, 'b, II: MRBIterator<Item = T>, I: AsyncIterator<'buf, I = II>, T>(
                    s: &mut I,
                    count: &mut usize,
                ) -> Option<MutableSlice<'b, T>> {
                    s.inner_mut().get_workable_slice_multiple_of(*count)
                }

                MRBFuture {
                    iter: self,
                    p: Some(count),
                    f_r: Some(f),
                    f_m: None,
                    phantom: PhantomData,
                }
            }

            pub unsafe fn advance(&mut self, count: usize) {
                unsafe {
                    self.inner.advance(count);
                }
                self.wake_next();
            }
        };
    }

    pub(crate) use gen_common_futs_fn;
}
