use core::mem::transmute;
use core::num::NonZeroUsize;
use core::slice;

use crate::iterators::{cons_alive, cons_index, private_impl, prod_alive, prod_index, public_impl};
use crate::iterators::iterator_trait::{MRBIterator, PrivateMRBIterator};
use crate::iterators::sync_iterators::detached_work_iter::DetachedWorkIter;
use crate::ring_buffer::storage::storage_trait::Storage;
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, IterManager, MutRB};
use crate::ring_buffer::wrappers::buf_ref::BufRef;
use crate::ring_buffer::wrappers::unsafe_sync_cell::UnsafeSyncCell;

/// Returned by slice-specialised functions.
/// # Fields:
/// - 1: head
/// - 2: tail
pub type WorkableSlice<'a, T> = (&'a mut [T], &'a mut [T]);

#[doc = r##"
Iterator used to mutate elements in-place.

<div class="warning">

This iterator returns mutable references to data stored within the buffer.
Thuss stated in the docs below, [`Self::advance`] has to be called when done with the mutation
in order to move the iterator.
</div>

[`Self::advance`] updates a global iterator, which is read by the consumer to decide if it can move on.
To avoid this [`DetachedWorkIter`] can be obtained by calling [`Self::detach`].
"##]

pub struct WorkIter<'buf, B: MutRB> {
    pub(crate) index: usize,
    pub(crate) cached_avail: usize,
    pub(crate) buf_len: NonZeroUsize,
    pub(crate) buffer: BufRef<'buf, B>,
}

unsafe impl<'buf, B: ConcurrentRB + MutRB<Item = T>, T> Send for WorkIter<'buf, B> {}

impl<'buf, B: MutRB + IterManager> Drop for WorkIter<'buf, B> {
    fn drop(&mut self) {
        self.buffer.set_work_alive(false);
    }
}

impl<'buf, B: MutRB<Item = T>, T> PrivateMRBIterator<T> for WorkIter<'buf, B> {
    #[inline(always)]
    fn set_atomic_index(&self, index: usize) {
        self.buffer.set_work_index(index);
    }

    #[inline(always)]
    fn succ_index(&self) -> usize {
        self.buffer.prod_index()
    }

    private_impl!(); }

impl<'buf, B: MutRB<Item = T>, T> MRBIterator<T> for WorkIter<'buf, B> {
    #[inline]
    fn available(&mut self) -> usize {
        let succ_idx = self.succ_index();

        unsafe {
            self.cached_avail = match self.index <= succ_idx {
                true => succ_idx.unchecked_sub(self.index),
                false => self.buf_len.get().unchecked_sub(self.index).unchecked_add(succ_idx)
            };
        }

        self.cached_avail
    }

    public_impl!();
}

impl<'buf, B: MutRB<Item = T>, T> WorkIter<'buf, B> {
    prod_alive!();
    cons_alive!();
    prod_index!();
    cons_index!();

    pub(crate) fn new(value: BufRef<'buf, B>) -> WorkIter<'buf, B> {
        Self {
            index: 0,
            buf_len: NonZeroUsize::new(value.inner_len()).unwrap(),
            buffer: value,
            cached_avail: 0,
        }
    }

    /// Detaches the iterator yielding a [`DetachedWorkIter`].
    #[inline]
    pub fn detach(self) -> DetachedWorkIter<'buf, B> {
        DetachedWorkIter::from_work(self)
    }

    /// Resets the index of the iterator. I.e., moves the iterator to the location occupied by its successor.
    #[inline]
    pub fn reset_index(&mut self) {
        let new_idx = self.succ_index();
        self.index = new_idx;
        self.set_atomic_index(new_idx);
    }

    /// Returns a mutable references to the current value.
    ///
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn get_workable<'a>(&mut self) -> Option<&'a mut T> {
        self.next_ref_mut()
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to `count`.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn get_workable_slice_exact<'a>(&mut self, count: usize) -> Option<WorkableSlice<'a, T>> {
        self.next_chunk_mut(count)
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to [`Self::available()`].
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn get_workable_slice_avail<'a>(&mut self) -> Option<WorkableSlice<'a, T>> {
        match self.available() {
            0 => None,
            avail => self.get_workable_slice_exact(avail)
        }
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to the
    /// maximum multiple of `modulo`.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn get_workable_slice_multiple_of<'a>(&mut self, rhs: usize) -> Option<WorkableSlice<'a, T>> {
        let avail = self.available();
        
        match avail - avail % rhs {
            0 => None,
            avail => self.get_workable_slice_exact(avail)
        }
    }
}