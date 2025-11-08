use crate::Storage;
use crate::iterators::async_iterators::async_macros::gen_common_futs_fn;
use crate::iterators::async_iterators::{AsyncIterator, MRBFuture};
use crate::iterators::iterator_trait::MutableSlice;
use crate::iterators::util_macros::delegate;
use crate::ring_buffer::variants::async_rb::AsyncMutRingBuf;
use crate::ring_buffer::wrappers::buf_ref::BufRef;
#[allow(unused_imports)]
use crate::{MRBIterator, iterators::WorkIter};
use core::marker::PhantomData;
use core::task::Waker;

#[doc = r##"
Async version of [`WorkIter`].
"##]
pub struct AsyncWorkIter<'buf, S: Storage> {
    pub(crate) inner: WorkIter<'buf, AsyncMutRingBuf<S>>,
    waker: BufRef<'buf, AsyncMutRingBuf<S>>,
}
unsafe impl<S: Storage> Send for AsyncWorkIter<'_, S> {}

impl<'buf, S: Storage> AsyncIterator<'buf> for AsyncWorkIter<'buf, S> {
    type I = WorkIter<'buf, AsyncMutRingBuf<S>>;
    type S = S;

    fn register_waker(&self, waker: &Waker) {
        (*self.waker).work_waker.register(waker);
    }

    fn take_waker(&self) -> Option<Waker> {
        (*self.waker).work_waker.take()
    }

    fn wake_next(&self) {
        (*self.waker).cons_waker.wake()
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

impl<'buf, S: Storage<Item = T>, T> AsyncWorkIter<'buf, S> {
    gen_common_futs_fn!( 'buf );
    delegate!(WorkIter, pub fn reset_index(&(mut) self));
}
