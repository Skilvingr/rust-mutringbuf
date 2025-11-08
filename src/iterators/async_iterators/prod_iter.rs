use core::marker::PhantomData;
use core::task::Waker;

use crate::Storage;
use crate::iterators::ProdIter;
use crate::iterators::async_iterators::async_macros::gen_common_futs_fn;
use crate::iterators::async_iterators::{AsyncIterator, MRBFuture};
use crate::iterators::iterator_trait::MRBIterator;
use crate::iterators::iterator_trait::MutableSlice;
use crate::ring_buffer::variants::async_rb::AsyncMutRingBuf;
use crate::ring_buffer::wrappers::buf_ref::BufRef;

#[doc = r##"
Async version of [`ProdIter`].
"##]
pub struct AsyncProdIter<'buf, S: Storage, const W: bool> {
    inner: ProdIter<'buf, AsyncMutRingBuf<S>>,
    waker: BufRef<'buf, AsyncMutRingBuf<S>>,
}
unsafe impl<S: Storage, const W: bool> Send for AsyncProdIter<'_, S, W> {}

impl<'buf, S: Storage, const W: bool> AsyncIterator<'buf> for AsyncProdIter<'buf, S, W> {
    type I = ProdIter<'buf, AsyncMutRingBuf<S>>;
    type S = S;

    fn register_waker(&self, waker: &Waker) {
        (*self.waker).prod_waker.register(waker);
    }

    fn take_waker(&self) -> Option<Waker> {
        (*self.waker).prod_waker.take()
    }

    fn wake_next(&self) {
        if W {
            (*self.waker).work_waker.wake()
        } else {
            (*self.waker).cons_waker.wake()
        }
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

impl<'buf, S: Storage<Item = T> + 'buf, T, const W: bool> AsyncProdIter<'buf, S, W> {
    gen_common_futs_fn!( 'buf );

    /// Async version of [`ProdIter::push`].
    pub fn push<'b>(&'b mut self, item: T) -> MRBFuture<'buf, 'b, Self, T, (), false> {
        #[inline]
        fn f<'buf, S: Storage<Item = T> + 'buf, T, const W: bool>(
            s: &mut AsyncProdIter<'buf, S, W>,
            item: T,
        ) -> Result<(), T> {
            s.inner_mut().push(item)
        }

        MRBFuture {
            iter: self,
            p: Some(item),
            f_r: None,
            f_m: Some(f),
            phantom: PhantomData,
        }
    }

    /// Async version of [`ProdIter::push_slice`].
    pub fn push_slice<'b>(
        &'b mut self,
        slice: &'b [T],
    ) -> MRBFuture<'buf, 'b, Self, &'b [T], (), true>
    where
        T: Copy,
    {
        #[inline]
        fn f<'buf, 'b, S: Storage<Item = T> + 'buf, T: Copy, const W: bool>(
            s: &mut AsyncProdIter<'buf, S, W>,
            slice: &mut &[T],
        ) -> Option<()> {
            let ret = s.inner_mut().push_slice(slice);
            s.wake_next();
            ret
        }

        MRBFuture {
            iter: self,
            p: Some(slice),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ProdIter::push_slice_clone`].
    pub fn push_slice_clone<'b>(
        &'b mut self,
        slice: &'b [T],
    ) -> MRBFuture<'buf, 'b, Self, &'b [T], (), true>
    where
        T: Clone,
    {
        #[inline]
        fn f<'buf, S: Storage<Item = T> + 'buf, T: Clone, const W: bool>(
            s: &mut AsyncProdIter<'buf, S, W>,
            slice: &mut &[T],
        ) -> Option<()> {
            s.inner_mut().push_slice_clone(slice)
        }

        MRBFuture {
            iter: self,
            p: Some(slice),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ProdIter::get_next_item_mut`].
    /// # Safety
    /// Same as [`ProdIter::get_next_item_mut`].
    pub unsafe fn get_next_item_mut<'b>(
        &'b mut self,
    ) -> MRBFuture<'buf, 'b, Self, (), &'b mut T, true> {
        #[inline]
        fn f<'buf, 'b, S: Storage<Item = T> + 'buf, T, const W: bool>(
            s: &mut AsyncProdIter<'buf, S, W>,
            _: &mut (),
        ) -> Option<&'b mut T> {
            unsafe { s.inner_mut().get_next_item_mut() }
        }

        MRBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ProdIter::get_next_item_mut_init`].
    pub fn get_next_item_mut_init<'b>(&'b mut self) -> MRBFuture<'buf, 'b, Self, (), *mut T, true> {
        #[inline]
        fn f<'buf, S: Storage<Item = T> + 'buf, T, const W: bool>(
            s: &mut AsyncProdIter<'buf, S, W>,
            _: &mut (),
        ) -> Option<*mut T> {
            s.inner_mut().get_next_item_mut_init()
        }

        MRBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ProdIter::get_next_slices_mut`].
    /// # Safety
    /// See above.
    pub unsafe fn get_next_slices_mut<'b>(
        &'b mut self,
        count: usize,
    ) -> MRBFuture<'buf, 'b, Self, usize, MutableSlice<'b, T>, true> {
        #[inline]
        fn f<'buf, 'b, S: Storage<Item = T> + 'buf, T, const W: bool>(
            s: &mut AsyncProdIter<'buf, S, W>,
            count: &mut usize,
        ) -> Option<MutableSlice<'b, T>> {
            unsafe { s.inner_mut().get_next_slices_mut(*count) }
        }

        MRBFuture {
            iter: self,
            p: Some(count),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }
}
