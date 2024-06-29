use crate::iterators::sync_iterators::work_iter::WorkableSlice;
use crate::iterators::iterator_trait::{MRBIterator, PrivateMRBIterator};
use crate::iterators::util_macros::delegate;
use crate::iterators::util_macros::muncher;
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

pub struct DetachedWorkIter<'buf, B: MutRB> {
    inner: WorkIter<'buf, B>
}

unsafe impl<'buf, B: ConcurrentRB + MutRB<Item = T>, T> Send for DetachedWorkIter<'buf, B> {}


impl<'buf, B: MutRB<Item = T>, T> DetachedWorkIter<'buf, B> {
    /// Creates a [`Self`] from a [`WorkIter`].
    #[inline]
    pub(crate) fn from_work(work: WorkIter<'buf, B>) -> DetachedWorkIter<'buf, B> {
        Self {
            inner: work
        }
    }

    /// Attaches the iterator, yielding a [`WorkIter`].
    #[inline]
    pub fn attach(self) -> WorkIter<'buf, B> {
        self.sync_index();

        self.inner
    }


    delegate!(WorkIter (inline), pub fn available(&(mut) self) -> usize);
    delegate!(WorkIter (inline), pub fn wait_for(&(mut) self, count: usize));
    delegate!(WorkIter (inline), pub fn index(&self) -> usize);
    delegate!(WorkIter (inline), pub fn buf_len(&self) -> usize);

    /// Sets local index.
    ///
    /// # Safety
    /// Index must always be between consumer and producer.
    #[inline(always)]
    pub unsafe fn set_index(&mut self, index: usize) {
        self.inner.index = index;
    }

    /// Resets the *local* index of the iterator. I.e., moves the iterator to the location occupied by its successor.
    /// To sync with the atomic index, use [`Self::sync_index`].
    #[inline]
    pub fn reset_index(&mut self) {
        let new_idx = self.inner.succ_index();
        self.inner.index = new_idx;
    }

    /// Advances the iterator as in [`WorkIter::available()`], but does not modify the atomic counter,
    /// making the change local.
    ///
    /// # Safety
    /// See [`WorkIter::advance`].
    #[inline]
    pub unsafe fn advance(&mut self, count: usize) {
        self.inner.index = match self.inner.index + count >= self.inner.buf_len.get() {
            true => self.inner.index + count - self.inner.buf_len.get(),
            false => self.inner.index + count
        };
    }

    /// Goes back, wrapping if necessary.
    ///
    /// # Safety
    /// Index must always be between consumer and producer.
    pub unsafe fn go_back(&mut self, count: usize) {
        self.inner.index = match self.inner.index < count {
            true => self.inner.buf_len.get() - (count - self.inner.index),
            false => self.inner.index - count
        };
    }

    delegate!(WorkIter (inline), pub fn is_prod_alive(&self) -> bool);
    delegate!(WorkIter (inline), pub fn is_cons_alive(&self) -> bool);
    delegate!(WorkIter (inline), pub fn prod_index(&self) -> usize);
    delegate!(WorkIter (inline), pub fn cons_index(&self) -> usize);
    delegate!(WorkIter (inline), pub fn get_workable(&(mut) self) -> Option<&mut T>);
    delegate!(WorkIter (inline), pub fn get_workable_slice_exact(&(mut) self, count: usize) -> Option<WorkableSlice<'_, T>>);
    delegate!(WorkIter (inline), pub fn get_workable_slice_avail(&(mut) self) -> Option<WorkableSlice<'_, T>>);
    delegate!(WorkIter (inline), pub fn get_workable_slice_multiple_of(&(mut) self, rhs: usize) -> Option<WorkableSlice<'_, T>>);

    /// Synchronises the underlying atomic index with the local index. I.e. let the consumer iterator
    /// advance.
    #[inline(always)]
    pub fn sync_index(&self) {
        self.inner.set_atomic_index(self.inner.index);
    }
}