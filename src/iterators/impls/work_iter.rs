use core::cell::UnsafeCell;
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
/// - 3: accumulator
pub type WorkableSlice<'a, T, A> = ((&'a mut [T], &'a mut [T]), &'a mut A);

#[doc = r##"
Iterator used to mutate elements in-place.

<div class="warning">

This iterator returns mutable references to data stored within the buffer.
Thus, as stated in the docs below, [`Self::advance`] has to be called when done with the mutation
in order to move the iterator.
</div>

[`Self::advance`] updates a global iterator, which is read by the consumer to decide if it can move on.
To avoid this, a [`DetachedWorkIter`] can be obtained by calling [`Self::detach`].
"##]

pub struct WorkIter<B: MutRB<T>, T, A> {
    pub(crate) index: usize,

    pub(crate) buf_len: usize,
    pub(crate) buffer: BufRef<B>,

    cached_avail: usize,

    accumulator: UnsafeCell<A>,

    _phantom: PhantomData<T>
}

unsafe impl<B: ConcurrentRB + MutRB<T>, T, A> Send for WorkIter<B, T, A> {}

impl<B: MutRB<T> + IterManager, T, A> Drop for WorkIter<B, T, A> {
    fn drop(&mut self) {
        self.buffer.set_work_alive(false);
    }
}

impl<B: MutRB<T>, T, A> PrivateMRBIterator<T> for WorkIter<B, T, A> {
    #[inline]
    fn set_index(&self, index: usize) {
        self.buffer.set_work_index(index);
    }

    #[inline]
    fn succ_index(&self) -> usize {
        self.buffer.prod_index()
    }

    private_impl!(); }

impl<B: MutRB<T>, T, A> MRBIterator<T> for WorkIter<B, T, A> {
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

impl<B: MutRB<T>, T, A> WorkIter<B, T, A> {
    prod_alive!();
    cons_alive!();

    pub(crate) fn new(value: BufRef<B>, accumulator: A) -> WorkIter<B, T, A> {
        Self {
            index: 0,
            cached_avail: 0,

            buf_len: value.inner_len(),
            buffer: value,
            accumulator: accumulator.into(),

            _phantom: PhantomData
        }
    }

    /// Detaches the iterator yielding a [`DetachedWorkIter`].
    #[inline]
    pub fn detach(self) -> DetachedWorkIter<B, T, A> {
        DetachedWorkIter::from_work(self)
    }

    /// Resets the index of the iterator. I.e., moves the iterator to the location occupied by its successor.
    #[inline]
    pub fn reset_index(&mut self) {
        let new_idx = self.succ_index();
        self.index = new_idx;
        self.set_index(new_idx);
    }

    /// Returns a tuple containing mutable references to actual value and accumulator.
    ///
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn get_workable(&mut self) -> Option<(&mut T, &mut A)> {
        let bt = self.accumulator.get();

        unsafe { self.next_ref_mut().map(|v| (v, &mut *bt)) }
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to `count`, along
    /// with accumulator.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn get_workable_slice_exact(&mut self, count: usize) -> Option<WorkableSlice<T, A>> {
        let bt = self.accumulator.get();

        unsafe { self.next_chunk_mut(count).map(|x| (x, &mut *bt)) }
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to [`Self::available()`], along
    /// with accumulator.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn get_workable_slice_avail(&mut self) -> Option<WorkableSlice<T, A>> {
        let avail = self.available();
        if avail > 0 { self.get_workable_slice_exact(avail) } else { None }
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to the
    /// maximum multiple of `modulo`, along with accumulator.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn get_workable_slice_multiple_of(&mut self, rhs: usize) -> Option<WorkableSlice<T, A>> {
        let avail = self.available();
        let avail = avail - avail % rhs;
        if avail > 0 { self.get_workable_slice_exact(avail) } else { None }
    }
}