use core::marker::PhantomData;
use core::mem::transmute;
use core::slice;

use crate::iterators::{cons_alive, private_impl, prod_alive, public_impl};
use crate::iterators::impls::detached_work_iter::DetachedWorkIter;
use crate::iterators::iterator_trait::{MRBIterator, PrivateMRBIterator};
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

pub struct WorkIter<B: MutRB<T>, T> {
    pub(crate) index: usize,

    pub(crate) buf_len: usize,
    pub(crate) buffer: BufRef<B>,

    cached_avail: usize,
    
    _phantom: PhantomData<T>
}

unsafe impl<B: ConcurrentRB + MutRB<T>, T> Send for WorkIter<B, T> {}

impl<B: MutRB<T> + IterManager, T> Drop for WorkIter<B, T> {
    fn drop(&mut self) {
        self.buffer.set_work_alive(false);
    }
}

impl<B: MutRB<T>, T> PrivateMRBIterator<T> for WorkIter<B, T> {
    #[inline]
    fn set_atomic_index(&self, index: usize) {
        self.buffer.set_work_index(index);
    }

    #[inline]
    fn succ_index(&self) -> usize {
        self.buffer.prod_index()
    }

    private_impl!(); }

impl<B: MutRB<T>, T> MRBIterator<T> for WorkIter<B, T> {
    #[inline]
    fn available(&mut self) -> usize {
        let succ_idx = self.succ_index();

        self.cached_avail = match self.index <= succ_idx {
            true => succ_idx - self.index,
            false => self.buf_len - self.index + succ_idx
        };

        self.cached_avail
    }

    public_impl!();
}

impl<B: MutRB<T>, T> WorkIter<B, T> {
    prod_alive!();
    cons_alive!();

    pub(crate) fn new(value: BufRef<B>) -> WorkIter<B, T> {
        Self {
            index: 0,
            cached_avail: 0,

            buf_len: value.inner_len(),
            buffer: value,

            _phantom: PhantomData
        }
    }

    /// Detaches the iterator yielding a [`DetachedWorkIter`].
    #[inline]
    pub fn detach(self) -> DetachedWorkIter<B, T> {
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
    pub fn get_workable(&mut self) -> Option<&mut T> {
        self.next_ref_mut()
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to `count`.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn get_workable_slice_exact(&mut self, count: usize) -> Option<WorkableSlice<T>> {
        self.next_chunk_mut(count)
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to [`Self::available()`].
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn get_workable_slice_avail(&mut self) -> Option<WorkableSlice<T>> {
        let avail = self.available();
        if avail > 0 { self.get_workable_slice_exact(avail) } else { None }
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to the
    /// maximum multiple of `modulo`.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn get_workable_slice_multiple_of(&mut self, rhs: usize) -> Option<WorkableSlice<T>> {
        let avail = self.available();
        let avail = avail - avail % rhs;
        if avail > 0 { self.get_workable_slice_exact(avail) } else { None }
    }
}