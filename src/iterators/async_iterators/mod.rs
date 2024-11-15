#![cfg(any(feature = "async", doc))]

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use crate::iterators::async_iterators::detached::AsyncDetached;
use crate::{MRBIterator, MutRB};
use crate::iterators::util_macros::delegate;
use crate::iterators::util_macros::muncher;

pub mod prod_iter;
pub mod work_iter;
pub mod cons_iter;
pub mod detached;


pub trait AsyncIterator {
    type I: MRBIterator;
    type B: MutRB;

    fn register_waker(&mut self, waker: &Waker);
    
    fn inner(&self) -> &Self::I;
    fn inner_mut(&mut self) -> &mut Self::I;

    fn into_sync(self) -> Self::I;

    fn from_sync(iter: Self::I) -> Self;

    fn detach(self) -> AsyncDetached<Self, Self::B,> where Self: Sized {
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
    delegate!(MRBIterator, unsafe fn advance(&(mut) self, count: usize));
}

pub struct MRBFuture<'a, I, P, O, const R: bool>
where I: AsyncIterator {
    iter: &'a mut I,
    p: Option<P>,
    f_r: Option<fn(&mut I, &mut P) -> Option<O>>,
    f_m: Option<fn(&mut I, P) -> Result<O, P>>
}

impl<'a, I: AsyncIterator, P, O, const R: bool> Unpin for MRBFuture<'a, I, P, O, R> {}

impl<'a, I: AsyncIterator, P, O, const R: bool> Future for MRBFuture<'a, I, P, O, R> {
    type Output = Option<O>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut waker_registered = false;
        let f_r = self.f_r.take();
        let f_m = self.f_m.take();

        loop {
            let res = if R {
                let mut p = self.p.take().unwrap();
                let ret = f_r.as_ref().unwrap()(self.iter, &mut p);
                ret.ok_or(p)
            } else {
                let p = self.p.take().unwrap();
                f_m.as_ref().unwrap()(self.iter, p)
            };
            
            match res {
                Ok(r) => { break Poll::Ready(Some(r)); }
                Err(p) => {
                    self.f_r = f_r;
                    self.f_m = f_m;
                    self.p = Some(p);
                }
            }

            if waker_registered {
                break Poll::Pending;
            }
            self.iter.register_waker(cx.waker());
            waker_registered = true;
        }
    }
}

pub(crate) mod async_macros {
    
    macro_rules! waker_registerer {
        () => {
            fn register_waker(&mut self, waker: &Waker) {
                self.waker.get_or_insert(waker.clone()).clone_from(waker);
            }
        };
    }
    
    macro_rules! gen_common_futs_fn { ($(($CT: ty))*) => {
        /// Async version of [`MRBIterator::get_workable`].
        pub fn get_workable<'b>(&mut self) -> MRBFuture<Self, (), &'b mut T, true> {
            fn f<'b, II: MRBIterator<Item = T>, I: AsyncIterator<I = II>, T>(s: &mut I, _: &mut ()) -> Option<&'b mut T> {
                s.inner_mut().get_workable()
            }
    
            MRBFuture {
                iter: self,
                p: Some(()),
                f_r: Some(f),
                f_m: None
            }
        }
    
        /// Async version of [`MRBIterator::get_workable_slice_exact`].
        pub fn get_workable_slice_exact<'b>(&mut self, count: usize) -> MRBFuture<Self, usize, (&'b mut [T], &'b mut [T]), true> {
            fn f<'b, II: MRBIterator<Item = T>, I: AsyncIterator<I = II>, T>(s: &mut I, count: &mut usize) -> Option<(&'b mut [T], &'b mut [T])> {
                s.inner_mut().get_workable_slice_exact(*count)
            }
    
            MRBFuture {
                iter: self,
                p: Some(count),
                f_r: Some(f),
                f_m: None
            }
        }
    
        /// Async version of [`MRBIterator::get_workable_slice_avail`].
        pub fn get_workable_slice_avail<'b>(&mut self) -> MRBFuture<Self, (), (&'b mut [T], &'b mut [T]), true> {
            fn f<'b, II: MRBIterator<Item = T>, I: AsyncIterator<I = II>, T>(s: &mut I, _: &mut ()) -> Option<(&'b mut [T], &'b mut [T])> {
                s.inner_mut().get_workable_slice_avail()
            }
    
            MRBFuture {
                iter: self,
                p: Some(()),
                f_r: Some(f),
                f_m: None
            }
        }
    
        /// Async version of [`MRBIterator::get_workable_slice_multiple_of`].
        pub fn get_workable_slice_multiple_of<'b>(&mut self, count: usize) -> MRBFuture<Self, usize, (&'b mut [T], &'b mut [T]), true> {
            fn f<'b, II: MRBIterator<Item = T>, I: AsyncIterator<I = II>, T>(s: &mut I, count: &mut usize) -> Option<(&'b mut [T], &'b mut [T])> {
                s.inner_mut().get_workable_slice_multiple_of(*count)
            }
    
            MRBFuture {
                iter: self,
                p: Some(count),
                f_r: Some(f),
                f_m: None
            }
        }
    }}

    pub(crate) use { gen_common_futs_fn, waker_registerer };
}