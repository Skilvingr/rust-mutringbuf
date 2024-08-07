use crate::AsyncWorkIter;
#[allow(unused_imports)]
use crate::DetachedWorkIter;
use crate::iterators::async_iterators::work_iter::GetWorkableFuture;
use crate::iterators::async_iterators::work_iter::GetWorkableSliceAvailFuture;
use crate::iterators::async_iterators::work_iter::GetWorkableSliceExactFuture;
use crate::iterators::async_iterators::work_iter::GetWorkableSliceMultipleOfFuture;
use crate::iterators::iterator_trait::PrivateMRBIterator;
use crate::iterators::util_macros::delegate;
use crate::iterators::util_macros::muncher;
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, MutRB};

#[doc = r##"
Async version of [`DetachedWorkIter`].
"##]

pub struct AsyncDetachedWorkIter<'buf, B: MutRB> {
    inner: AsyncWorkIter<'buf, B>,
}

unsafe impl<'buf, B: ConcurrentRB + MutRB<Item = T>, T> Send for AsyncDetachedWorkIter<'buf, B> {}

impl<'buf, B: MutRB<Item = T>, T> AsyncDetachedWorkIter<'buf, B> {
    /// Creates [`Self`] from an [`AsyncWorkIter`].
    pub(crate) fn from_work(work: AsyncWorkIter<'buf, B>) -> AsyncDetachedWorkIter<'buf, B> {
        Self {
            inner: work,
        }
    }

    pub fn from_sync(work: DetachedWorkIter<'buf, B>) -> AsyncDetachedWorkIter<'buf, B> {
        Self {
            inner: AsyncWorkIter::from_sync(work.attach()),
        }
    }

    pub fn into_sync(self) -> DetachedWorkIter<'buf, B> {
        self.inner.inner.detach()
    }

    /// Same as [`DetachedWorkIter::attach`].
    pub fn attach(self) -> AsyncWorkIter<'buf, B> {
        self.sync_index();
        self.inner
    }

    delegate!(DetachedWorkIter, pub fn is_prod_alive(&self) -> bool);
    delegate!(DetachedWorkIter, pub fn is_cons_alive(&self) -> bool);
    delegate!(DetachedWorkIter, pub fn prod_index(&self) -> usize);
    delegate!(DetachedWorkIter, pub fn cons_index(&self) -> usize);
    delegate!(DetachedWorkIter, pub fn index(&self) -> usize);
    delegate!(DetachedWorkIter, pub fn available(&(mut) self) -> usize);

    delegate!(DetachedWorkIter, pub fn get_workable(&(mut) self) -> GetWorkableFuture<'buf, '_, B, T>);
    delegate!(DetachedWorkIter, pub fn get_workable_slice_exact(&(mut) self, count: usize) -> GetWorkableSliceExactFuture<'buf, '_, B, T>);
    delegate!(DetachedWorkIter, pub fn get_workable_slice_avail(&(mut) self) -> GetWorkableSliceAvailFuture<'buf, '_, B, T>);
    delegate!(DetachedWorkIter, pub fn get_workable_slice_multiple_of(&(mut) self, rhs: usize) -> GetWorkableSliceMultipleOfFuture<'buf, '_, B, T>);

    /// Same as [`DetachedWorkIter::sync_index`].
    pub fn sync_index(&self) {
        self.inner.inner.set_atomic_index(self.inner.inner.index)
    }

    /// Same as [`DetachedWorkIter::advance`].
    ///
    /// # Safety
    /// Same as [`DetachedWorkIter::advance`].
    pub unsafe fn advance(&mut self, count: usize) {
        self.inner.inner.advance_local(count);
    }

    /// Same as [`DetachedWorkIter::go_back`].
    ///
    /// # Safety
    /// Same as [`DetachedWorkIter::go_back`].
    pub unsafe fn go_back(&mut self, count: usize) {
        self.inner.inner.index = match self.inner.inner.index < count {
            true => self.inner.inner.buf_len.get() - (count - self.inner.inner.index),
            false => self.inner.inner.index - count
        };

        self.inner.inner.cached_avail = self.inner.inner.cached_avail.unchecked_add(count);
    }
}