use core::marker::PhantomData;
use crate::iterators::async_iterators::AsyncIterator;

use crate::{MRBIterator, MutRB};
use crate::iterators::iterator_trait::PrivateMRBIterator;
#[allow(unused_imports)]
use crate::iterators::sync_iterators::detached::Detached;


#[doc = r##"
Async version of [`Detached`].
"##]

pub struct AsyncDetached<I: AsyncIterator, B: MutRB> {
    inner: I,
    phantom_data: PhantomData<B>
}

unsafe impl<I: AsyncIterator, B: MutRB> Send for AsyncDetached<I, B> {}

impl<B: MutRB<Item = T>, T, I: AsyncIterator> AsyncDetached<I, B> {
       
    /// Creates [`Self`] from an [`AsyncWorkIter`].
    pub(crate) fn from_iter(iter: I) -> AsyncDetached<I, B> {
        Self {
            inner: iter,
            phantom_data: PhantomData
        }
    }

    /// Same as [`Detached::attach`].
    pub fn attach(self) -> I {
        self.sync_index();
        self.inner
    }
    
    /// Same as [`Detached::sync_index`].
    pub fn sync_index(&self) {
        self.inner.inner().set_atomic_index(self.inner.index())
    }

    /// Same as [`Detached::advance`].
    ///
    /// # Safety
    /// Same as [`Detached::advance`].
    pub unsafe fn advance(&mut self, count: usize) {
        self.inner.inner_mut().advance_local(count);
    }

    /// Same as [`Detached::go_back`].
    ///
    /// # Safety
    /// Same as [`Detached::go_back`].
    pub unsafe fn go_back(&mut self, count: usize) {
        let idx = self.inner.inner_mut().index();
        let buf_len = self.inner.inner_mut().buf_len();
        
        self.inner.inner_mut().set_local_index(
            match idx < count {
                true => buf_len - (count - idx),
                false => idx - count
            }
        );

        let avail = self.inner.inner_mut().cached_avail();
        self.inner.inner_mut().set_cached_avail(avail.unchecked_add(count));
    }
}