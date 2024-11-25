use core::mem::transmute;
use core::slice;

use crate::iterators::{private_impl, public_impl};
use crate::iterators::iterator_trait::{MRBIterator, PrivateMRBIterator};
#[allow(unused_imports)]
use crate::iterators::sync_iterators::detached::Detached;
use crate::ring_buffer::storage::storage_trait::Storage;
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, IterManager, MutRB};
use crate::ring_buffer::wrappers::buf_ref::BufRef;
use crate::ring_buffer::wrappers::unsafe_sync_cell::UnsafeSyncCell;

#[doc = r##"
Iterator used to mutate elements in-place.

<div class="warning">

This iterator returns mutable references to data stored within the buffer.
Thus, as stated in the docs below, [`Self::advance`] has to be called when done with the mutation
in order to move the iterator.
</div>

[`Self::advance`] updates a global iterator, which is read by the consumer to decide if it can move on.
To avoid this [`Detached`] can be obtained by calling [`Self::detach`].
"##]

pub struct WorkIter<'buf, B: MutRB> {
    pub(crate) index: usize,
    pub(crate) cached_avail: usize,
    pub(crate) buffer: BufRef<'buf, B>,
}

unsafe impl<'buf, B: ConcurrentRB + MutRB<Item = T>, T> Send for WorkIter<'buf, B> {}

impl<'buf, B: MutRB + IterManager> Drop for WorkIter<'buf, B> {
    fn drop(&mut self) {
        self.buffer.set_work_alive(false);
    }
}

impl<'buf, B: MutRB<Item = T>, T> PrivateMRBIterator for WorkIter<'buf, B> {
    type PItem = T;
    
    #[inline]
    fn set_atomic_index(&self, index: usize) {
        self.buffer.set_work_index(index);
    }

    #[inline]
    fn succ_index(&self) -> usize {
        self.buffer.prod_index()
    }

    private_impl!();
}

impl<'buf, B: MutRB<Item = T>, T> MRBIterator for WorkIter<'buf, B> {
    type Item = T;
    
    #[inline]
    fn available(&mut self) -> usize {
        let succ_idx = self.succ_index();

        unsafe {
            self.cached_avail = match self.index <= succ_idx {
                true => succ_idx.unchecked_sub(self.index),
                false => self.buf_len().unchecked_sub(self.index).unchecked_add(succ_idx)
            };
        }

        self.cached_avail
    }

    public_impl!();
}

impl<'buf, B: MutRB<Item = T>, T> WorkIter<'buf, B> {
    pub(crate) fn new(value: BufRef<'buf, B>) -> WorkIter<'buf, B> {
        Self {
            index: 0,
            buffer: value,
            cached_avail: 0,
        }
    }

    /// Resets the index of the iterator. I.e., moves the iterator to the location occupied by its successor.
    #[inline]
    pub fn reset_index(&mut self) {
        let new_idx = self.succ_index();
        self.index = new_idx;
        self.set_atomic_index(new_idx);
    }
}