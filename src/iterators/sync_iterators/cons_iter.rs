use core::mem::transmute;
use core::num::NonZeroUsize;
use core::slice;

use crate::iterators::{private_impl, prod_alive, prod_index, public_impl, work_alive, work_index};
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

pub struct ConsIter<B: MutRB, const W: bool> {
    index: usize,
    buf_len: NonZeroUsize,
    buffer: BufRef<B>,

    cached_avail: usize,
}

unsafe impl<B: ConcurrentRB + MutRB<Item = T>, T, const W: bool> Send for ConsIter<B, W> {}

impl<B: MutRB + IterManager, const W: bool> Drop for ConsIter<B, W> {
    fn drop(&mut self) {
        self.buffer.set_cons_alive(false);
    }
}

impl<B: MutRB<Item = T>, T, const W: bool> PrivateMRBIterator<T> for ConsIter<B, W> {
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

impl<B: MutRB<Item = T>, T, const W: bool> MRBIterator<T> for ConsIter<B, W> {
    #[inline]
    fn available(&mut self) -> usize {
        let succ_idx = self.succ_index();

        self.cached_avail = match self.index <= succ_idx {
            true => succ_idx - self.index,
            false => self.buf_len.get() - self.index + succ_idx
        };

        self.cached_avail
    }

    public_impl!();
}

impl<B: MutRB<Item = T>, T, const W: bool> ConsIter<B, W> {
    prod_alive!();
    work_alive!();
    prod_index!();
    work_index!();
    
    pub(crate) fn new(value: BufRef<B>) -> Self {
        Self {
            index: 0,
            buf_len: NonZeroUsize::new(value.inner_len()).unwrap(),
            buffer: value,
            cached_avail: 0
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
    pub unsafe fn pop(&mut self) -> Option<T> {
        self.next()
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
        use crate::ConcurrentHeapRB;
        use super::*;

        const BUFFER_SIZE: usize = 100;
        
        let (mut prod, mut cons) = ConcurrentHeapRB::<u32>::default(BUFFER_SIZE + 1).split();

        assert_eq!(cons.cached_avail, 0);
        
        unsafe { prod.advance(10); }

        assert_eq!(cons.cached_avail, 0);
        
        cons.check(1);

        assert_eq!(cons.cached_avail, 10);
        
        unsafe { cons.advance(9); }

        assert_eq!(cons.cached_avail, 1);
    }
}