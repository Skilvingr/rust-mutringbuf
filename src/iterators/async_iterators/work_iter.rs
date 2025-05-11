use crate::iterators::async_iterators::async_macros::{gen_common_futs_fn, waker_registerer};
use crate::iterators::async_iterators::{AsyncIterator, MRBFuture};
use crate::iterators::iterator_trait::MutableSlice;
use crate::iterators::util_macros::delegate;
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, MutRB};
#[allow(unused_imports)]
use crate::{MRBIterator, iterators::WorkIter};
use core::task::Waker;

#[doc = r##"
Async version of [`WorkIter`].
"##]
pub struct AsyncWorkIter<'buf, B: MutRB> {
    pub(crate) inner: WorkIter<'buf, B>,
    waker: Option<Waker>
}
unsafe impl<B: ConcurrentRB + MutRB<Item = T>, T> Send for AsyncWorkIter<'_, B> {}

impl<'buf, B: MutRB<Item = T>, T> AsyncIterator for AsyncWorkIter<'buf, B> {
    type I = WorkIter<'buf, B>;
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

impl<B: MutRB<Item = T>, T> AsyncWorkIter<'_, B> {
    gen_common_futs_fn!();
    delegate!(WorkIter, pub fn reset_index(&(mut) self));
}