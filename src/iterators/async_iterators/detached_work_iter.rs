use crate::iterators::async_iterators::AsyncIterator;
#[allow(unused_imports)]
use crate::iterators::async_iterators::work_iter::GetWorkableFuture;
use crate::iterators::iterator_trait::PrivateMRBIterator;
use crate::iterators::util_macros::delegate;
use crate::iterators::util_macros::muncher;
use crate::MRBIterator;

#[doc = r##"
Async version of [`DetachedWorkIter`].
"##]

pub struct AsyncDetachedIter<I: AsyncIterator> {
    inner: I,
}

unsafe impl<I: AsyncIterator> Send for AsyncDetachedIter<I> {}

impl<I: AsyncIterator> AsyncDetachedIter<I> {
    /// Creates [`Self`] from an [`AsyncWorkIter`].
    pub(crate) fn from_iter(iter: I) -> AsyncDetachedIter<I> {
        Self {
            inner: iter,
        }
    }

    /// Same as [`DetachedWorkIter::attach`].
    pub fn attach(self) -> I {
        self.sync_index();
        self.inner
    }

    fn inner(&self) -> &I {
        &self.inner
    }
    fn inner_mut(&mut self) -> &mut I {
        &mut self.inner
    }
    
    delegate!(DetachedIter, pub fn is_prod_alive(&self) -> bool);
    delegate!(DetachedIter, pub fn is_cons_alive(&self) -> bool);
    delegate!(DetachedIter, pub fn prod_index(&self) -> usize);
    delegate!(DetachedIter, pub fn cons_index(&self) -> usize);
    delegate!(DetachedIter, pub fn index(&self) -> usize);
    delegate!(DetachedIter, pub fn available(&(mut) self) -> usize);

    // delegate!(DetachedIter, pub fn get_workable(&(mut) self) -> GetWorkableFuture<'buf, '_, B, T>);
    // delegate!(DetachedIter, pub fn get_workable_slice_exact(&(mut) self, count: usize) -> GetWorkableSliceExactFuture<'buf, '_, B, T>);
    // delegate!(DetachedIter, pub fn get_workable_slice_avail(&(mut) self) -> GetWorkableSliceAvailFuture<'buf, '_, B, T>);
    // delegate!(DetachedIter, pub fn get_workable_slice_multiple_of(&(mut) self, rhs: usize) -> GetWorkableSliceMultipleOfFuture<'buf, '_, B, T>);

    /// Same as [`DetachedWorkIter::sync_index`].
    pub fn sync_index(&self) {
        self.inner.inner().set_atomic_index(self.inner.index())
    }

    /// Same as [`DetachedWorkIter::advance`].
    ///
    /// # Safety
    /// Same as [`DetachedWorkIter::advance`].
    pub unsafe fn advance(&mut self, count: usize) {
        self.inner.inner_mut().advance_local(count);
    }

    /// Same as [`DetachedWorkIter::go_back`].
    ///
    /// # Safety
    /// Same as [`DetachedWorkIter::go_back`].
    pub unsafe fn go_back(&mut self, count: usize) {
        let idx = self.inner.inner_mut().index();
        let buf_len = self.inner.inner_mut().buf_len();
        
        self.inner.inner_mut().set_index(
            match idx < count {
                true => buf_len - (count - idx),
                false => idx - count
            }
        );

        let avail = self.inner.inner_mut().cached_avail();
        self.inner.inner_mut().set_cached_avail(avail.unchecked_add(count));
    }
}