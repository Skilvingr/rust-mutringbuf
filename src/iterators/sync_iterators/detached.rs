use crate::iterators::iterator_trait::{MRBIterator, MutableSlice};
use crate::iterators::util_macros::delegate;
use crate::iterators::util_macros::muncher;
#[allow(unused_imports)]
use crate::iterators::WorkIter;

#[doc = r##"
Detached iterator: does not update the atomic index when advancing.

This makes it possible to explore available data back and forth, putting other iterators on hold.

A typical use case of this structure is to search something amidst produced data, aligning the detached
iterator to a suitable index and then returning to a normal iterator.

This struct can only be created by [`detaching`](MRBIterator::detach) an iterator.

When done worker iterator can be re-obtained via [`Self::attach`].

Note that, in order to avoid buffer saturation, the global index can be synced with [`Self::sync_index`];
this synchronises indices making the consumer iterator able to move on.

<div class="warning">

As [`WorkIter`], this iterator returns mutable references to data stored within the buffer.
Thus, as stated in the docs written for the former, [`Self::advance`] has to be called when done with the mutation
in order to move the iterator.
</div>
"##]
pub struct Detached<I: MRBIterator> {
    inner: I,
}

unsafe impl<I: MRBIterator> Send for Detached<I> {}


impl<T, I: MRBIterator<Item = T>> Detached<I> {
    /// Creates a [`Self`] from an iterator.
    #[inline]
    pub(crate) fn from_iter(iter: I) -> Detached<I> {
        Self {
            inner: iter
        }
    }

    /// Attaches and yields the iterator.
    #[inline]
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


    delegate!(MRBIterator (inline), pub fn available(&(mut) self) -> usize);
    delegate!(MRBIterator (inline), pub fn wait_for(&(mut) self, count: usize));
    delegate!(MRBIterator (inline), pub fn index(&self) -> usize);
    delegate!(MRBIterator (inline), pub fn buf_len(&self) -> usize);

    /// Sets the *local* index. To sync the atomic index, use [`Self::sync_index`].
    ///
    /// # Safety
    /// Index must always be between consumer and producer.
    #[inline]
    pub unsafe fn set_index(&mut self, index: usize) {
        self.inner.set_local_index(index);
    }

    /// Resets the *local* index of the iterator. I.e., moves the iterator to the location occupied by its successor.
    /// To sync the atomic index, use [`Self::sync_index`].
    #[inline]
    pub fn reset_index(&mut self) {
        let new_idx = self.inner.succ_index();
        self.inner.set_local_index(new_idx);
    }

    /// Advances the iterator as in [`MRBIterator::advance()`], but does not modify the atomic counter,
    /// making the change local.
    ///
    /// # Safety
    /// See [`MRBIterator::advance`].
    #[inline]
    pub unsafe fn advance(&mut self, count: usize) {
        self.inner.advance_local(count);
    }

    /// Goes back, wrapping if necessary.
    ///
    /// # Safety
    /// Index must always be between consumer and producer.
    pub unsafe fn go_back(&mut self, count: usize) {
        let idx = self.inner.index();
        
        self.inner.set_local_index(
            match idx < count {
                true => self.inner.buf_len().unchecked_sub(count).unchecked_sub(idx),
                false => idx.unchecked_sub(count)
            }
        );

        let cached_avail = self.inner.cached_avail();
        self.inner.set_cached_avail(cached_avail.unchecked_add(count));
    }

    delegate!(MRBIterator (inline), pub fn is_prod_alive(&self) -> bool);
    delegate!(MRBIterator (inline), pub fn is_work_alive(&self) -> bool);
    delegate!(MRBIterator (inline), pub fn is_cons_alive(&self) -> bool);
    delegate!(MRBIterator (inline), pub fn prod_index(&self) -> usize);
    delegate!(MRBIterator (inline), pub fn work_index(&self) -> usize);
    delegate!(MRBIterator (inline), pub fn cons_index(&self) -> usize);
    
    delegate!(MRBIterator (inline), pub fn get_workable(&(mut) self) -> Option<&'_ mut T>);
    delegate!(MRBIterator (inline), pub fn get_workable_slice_exact(&(mut) self, count: usize) -> Option<MutableSlice<'_, T>>);
    delegate!(MRBIterator (inline), pub fn get_workable_slice_avail(&(mut) self) -> Option<MutableSlice<'_, T>>);
    delegate!(MRBIterator (inline), pub fn get_workable_slice_multiple_of(&(mut) self, rhs: usize) -> Option<MutableSlice<'_, T>>);

    /// Synchronises the underlying atomic index with the local index. I.e. let the consumer iterator
    /// advance.
    #[inline]
    pub fn sync_index(&self) {
        self.inner.set_atomic_index(self.inner.index());
    }
}