use crate::iterators::impls::work_iter::WorkableSlice;
use crate::iterators::iterator_trait::{MRBIterator, PrivateMRBIterator};
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, MutRB};
use crate::WorkIter;

#[doc = r##"
Same as [`WorkIter`], but does not update the atomic index when advancing.

This makes it possible to explore available data back and forth, putting the consumer iterator on hold.

A typical use case of this structure is to search something amidst produced dataligning the detached
iterator to a suitable indexnd then returning to a normal worker iterator.

This struct can only be created by [`detaching`](WorkIter::detach) a worker iterator.

When done worker iterator can be re-obtained via [`Self::attach`].

Note that, in order to avoid buffer saturationtomic index can be synced with [`Self::sync_index`];
this synchronises indices making the consumer iterator able to move on.

<div class="warning">

As [`WorkIter`], this iterator returns mutable references to data stored within the buffer.
Thuss stated in the docs written for the former, [`Self::advance`] has to be called when done with the mutation
in order to move the iterator.
</div>
"##]

pub struct DetachedWorkIter<B: MutRB<T>, T> {
    work_iter: WorkIter<B, T>
}

unsafe impl<B: ConcurrentRB + MutRB<T>, T> Send for DetachedWorkIter<B, T> {}


impl<B: MutRB<T>, T> DetachedWorkIter<B, T> {
    /// See [`WorkIter::available`].
    #[inline]
    pub fn available(&mut self) -> usize {
        self.work_iter.available()
    }

    /// See [`WorkIter::index`].
    #[inline]
    pub fn index(&self) -> usize {
        self.work_iter.index()
    }

    /// See [`WorkIter::buf_len`].
    #[inline]
    pub fn buf_len(&self) -> usize {
        self.work_iter.buf_len()
    }

    /// Sets local index.
    ///
    /// # Safety
    /// Index must always be between consumer and producer.
    #[inline]
    pub unsafe fn set_index(&mut self, index: usize) {
        self.work_iter.index = index;
    }

    /// Advances the iterator as in [`WorkIter::available()`], but does not modify the atomic counter,
    /// making the change local.
    ///
    /// # Safety
    /// See [`WorkIter::advance`].
    #[inline]
    pub unsafe fn advance(&mut self, count: usize) {
        self.work_iter.index = match self.work_iter.index + count >= self.work_iter.buf_len {
            true => self.work_iter.index + count - self.work_iter.buf_len,
            false => self.work_iter.index + count
        };
    }
}

impl<B: MutRB<T>, T> DetachedWorkIter<B, T> {
    /// Creates a [`Self`] from a [`WorkIter`].
    #[inline]
    pub(crate) fn from_work(work: WorkIter<B, T>) -> DetachedWorkIter<B, T> {
        Self {
            work_iter: work
        }
    }

    /// Attaches the iterator, yielding a [`WorkIter`].
    #[inline]
    pub fn attach(self) -> WorkIter<B, T> {
        self.sync_index();

        self.work_iter
    }

    /// Goes back, wrapping if necessary.
    ///
    /// # Safety
    /// Index must always be between consumer and producer.
    pub unsafe fn go_back(&mut self, count: usize) {
        self.work_iter.index = match self.work_iter.index < count {
            true => self.work_iter.buf_len - (count - self.work_iter.index),
            false => self.work_iter.index - count
        };
    }

    /// See [`WorkIter::is_prod_alive`].
    #[inline]
    pub fn is_prod_alive(&self) -> bool {
        self.work_iter.is_prod_alive()
    }

    /// See [`WorkIter::is_cons_alive`].
    #[inline]
    pub fn is_cons_alive(&self) -> bool {
        self.work_iter.is_cons_alive()
    }

    /// Synchronises the underlying atomic index with the local index. I.e. let the consumer iterator
    /// advance.
    #[inline]
    pub fn sync_index(&self) {
        self.work_iter.set_atomic_index(self.work_iter.index);
    }

    /// See [`WorkIter::get_workable`].
    #[inline]
    pub fn get_workable(&mut self) -> Option<&mut T> {
        self.work_iter.get_workable()
    }

    /// See [`WorkIter::get_workable_slice_exact`].
    #[inline]
    pub fn get_workable_slice_exact(&mut self, count: usize) -> Option<WorkableSlice<'_, T>> {
        self.work_iter.get_workable_slice_exact(count)
    }

    /// See [`WorkIter::get_workable_slice_avail`].
    #[inline]
    pub fn get_workable_slice_avail(&mut self) -> Option<WorkableSlice<'_, T>> {
        self.work_iter.get_workable_slice_avail()
    }

    /// See [`WorkIter::get_workable_slice_multiple_of`].
    #[inline]
    pub fn get_workable_slice_multiple_of(&mut self, rhs: usize) -> Option<WorkableSlice<'_, T>> {
        self.work_iter.get_workable_slice_multiple_of(rhs)
    }
}