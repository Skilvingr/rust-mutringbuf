use core::mem::transmute;
use core::num::NonZeroUsize;
use core::slice;

use crate::iterators::{private_impl, public_impl};
use crate::iterators::iterator_trait::{MRBIterator, PrivateMRBIterator};
#[allow(unused_imports)]
use crate::ProdIter;
use crate::ring_buffer::storage::storage_trait::Storage;
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, IterManager, MutRB};
use crate::ring_buffer::wrappers::buf_ref::BufRef;
use crate::ring_buffer::wrappers::unsafe_sync_cell::UnsafeSyncCell;

#[doc = r##"
Iterator used to pop data from the buffer.

When working with types which implement both [`Copy`] and [`Clone`] traits, `copy` methods should be
preferred over `clone` methods.
"##]

pub struct ConsIter<'buf, B: MutRB, const W: bool> {
    index: usize,
    cached_avail: usize,
    buf_len: NonZeroUsize,
    buffer: BufRef<'buf, B>,
}

unsafe impl<'buf, B: ConcurrentRB + MutRB<Item = T>, T, const W: bool> Send for ConsIter<'buf, B, W> {}

impl<'buf, B: MutRB + IterManager, const W: bool> Drop for ConsIter<'buf, B, W> {
    fn drop(&mut self) {
        self.buffer.set_cons_alive(false);
    }
}

impl<'buf, B: MutRB<Item = T>, T, const W: bool> PrivateMRBIterator for ConsIter<'buf, B, W> {
    type PItem = T;

    #[inline(always)]
    fn set_atomic_index(&self, index: usize) {
        self.buffer.set_cons_index(index);
    }

    #[inline(always)]
    fn succ_index(&self) -> usize {
        if W {
            self.buffer.work_index()
        } else {
            self.buffer.prod_index()
        }
    }

    private_impl!();
}

impl<'buf, B: MutRB<Item = T>, T, const W: bool> MRBIterator for ConsIter<'buf, B, W> {
    type Item = T;

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

impl<'buf, B: MutRB<Item = T>, T, const W: bool> ConsIter<'buf, B, W> {
    pub(crate) fn new(value: BufRef<'buf, B>) -> Self {
        Self {
            index: 0,
            buf_len: NonZeroUsize::new(value.inner_len()).unwrap(),
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

    /// Returns a reference to an element.
    /// <div class="warning">
    ///
    /// Being this a reference, [`Self::advance()`] has to be called when done with the data
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn peek_ref<'a>(&mut self) -> Option<&'a T> {
        self.next_ref()
    }

    /// Returns a tuple of slice references, the sum of which with len equal to `count`.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the data
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn peek_slice<'a>(&mut self, count: usize) -> Option<(&'a [T], &'a [T])> {
        self.next_chunk(count)
    }

    /// Returns a tuple of slice references, the sum of which with len equal to available data.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the data
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn peek_available<'a>(&mut self) -> Option<(&'a [T], &'a [T])> {
        let avail = self.available();
        self.peek_slice(avail)
    }

    /// Tries to pop an element, moving it.
    /// # Safety
    /// This method moves items, so locations from which they are moved out are left uninitialised.
    /// These locations must be re-initialised used proper [`ProdIter`] methods (`*_init`) ones
    #[inline]
    pub unsafe fn pop_move(&mut self) -> Option<T> {
        self.next()
    }

    /// Tries to pop an element, duplicating it.
    /// # Safety
    /// This method acts like `ptr::read`: it duplicates the item by making a bitwise copy, ignoring whether it is `Copy`/`Clone` or not.
    /// So it is your responsibility to ensure that the data may indeed be duplicated.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.next_duplicate()
    }

    /// - Returns `Some(())`, copying next item into `dst`, if available.
    /// - Returns `None` doing nothing, otherwise.
    ///
    /// This method uses `copy` and should be preferred over `clone` version, if possible.
    /// <div class="warning">
    ///
    /// Unlike `peek*` methods, this one automatically advances the iterator.
    /// </div>
    #[inline]
    pub fn copy_item(&mut self, dst: &mut T) -> Option<()>
        where T: Copy
    {
        if let Some(v) = self.next_ref() {
            *dst = *v;

            unsafe { self.advance(1) };
            Some(())
        } else { None }
    }

    /// Same as [`Self::copy_item`], but uses `clone`, instead.
    /// <div class="warning">
    ///
    /// Unlike `peek*` methods, this one automatically advances the iterator.
    /// </div>
    #[inline]
    pub fn clone_item(&mut self, dst: &mut T) -> Option<()>
        where T: Clone
    {
        if let Some(v) = self.next_ref() {
            *dst = v.clone();

            unsafe { self.advance(1) };
            Some(())
        } else { None }
    }

    /// - Returns `Some(())`, filling `dst` slice with the next `dst.len()` values, if available.
    /// - Returns `None` doing nothing, otherwise.
    ///
    /// This method fills the slice using `copy` and should be preferred over `clone` version, if possible.
    /// <div class="warning">
    ///
    /// Unlike `peek*` methods, this one automatically advances the iterator.
    /// </div>
    #[inline]
    pub fn copy_slice(&mut self, dst: &mut [T]) -> Option<()>
        where T: Copy
    {
        if let Some((h, t)) = self.next_chunk(dst.len()) {
            unsafe {
                dst.get_unchecked_mut(..h.len()).copy_from_slice(h);
                dst.get_unchecked_mut(h.len()..).copy_from_slice(t);
            }
                
            unsafe { self.advance(dst.len()) };
            Some(())
        } else { None }
    }

    /// Same as [`Self::copy_slice`], but uses `clone`, instead.
    /// <div class="warning">
    ///
    /// Unlike `peek*` methods, this one automatically advances the iterator.
    /// </div>
    #[inline]
    pub fn clone_slice(&mut self, dst: &mut [T]) -> Option<()>
        where T: Clone
    {
        if let Some((h, t)) = self.next_chunk(dst.len()) {
            unsafe {
                dst.get_unchecked_mut(..h.len()).clone_from_slice(h);
                dst.get_unchecked_mut(h.len()..).clone_from_slice(t);
            }

            unsafe { self.advance(dst.len()) };
            Some(())
        } else { None }
    }
}

mod test {

    #[test]
    fn cached_avail() {
        use super::*;
        use crate::{ConcurrentStackRB, StackSplit};

        const BUFFER_SIZE: usize = 100;
        
        let mut buf = ConcurrentStackRB::<u32, { BUFFER_SIZE + 1 }>::default();
        let (mut prod, mut cons) = buf.split();
        
        assert_eq!(cons.cached_avail, 0);
        
        unsafe { prod.advance(10); }

        assert_eq!(cons.cached_avail, 0);
        
        cons.check(1);

        assert_eq!(cons.cached_avail, 10);
        
        unsafe { cons.advance(9); }

        assert_eq!(cons.cached_avail, 1);
    }
}