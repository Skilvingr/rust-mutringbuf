use core::marker::PhantomData;
use core::mem::transmute;
use core::slice;

use crate::iterators::{private_impl, prod_alive, public_impl, work_alive};
use crate::iterators::iterator_trait::{Iterator, PrivateIterator};
use crate::ring_buffer::storage::storage_trait::Storage;
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, IterManager, MutRB};
use crate::ring_buffer::wrappers::buf_ref::BufRef;
use crate::ring_buffer::wrappers::unsafe_sync_cell::UnsafeSyncCell;

#[doc = r##"
Iterator used to pop data from the buffer.

When working with types which implement both [`Copy`] and [`Clone`] traits, `copy` methods should be
preferred over `clone` methods.
"##]

pub struct ConsIter<B: MutRB<T>, T, const W: bool> {
    index: usize,
    buf_len: usize,
    cached_avail: usize,

    buffer: BufRef<B>,

    _phantom: PhantomData<T>
}

unsafe impl<B: ConcurrentRB + MutRB<T>, T, const W: bool> Send for ConsIter<B, T, W> {}

impl<B: MutRB<T> + IterManager, T, const W: bool> Drop for ConsIter<B, T, W> {
    fn drop(&mut self) {
        self.buffer.set_cons_alive(false);
    }
}

impl<B: MutRB<T>, T, const W: bool> PrivateIterator<T> for ConsIter<B, T, W> {
    #[inline]
    fn set_index(&self, index: usize) {
        self.buffer.set_cons_index(index);
    }

    #[inline]
    fn succ_index(&self) -> usize {
        if W {
            self.buffer.work_index()
        } else {
            self.buffer.prod_index()
        }
    }

    private_impl!();
}

impl<B: MutRB<T>, T, const W: bool> Iterator<T> for ConsIter<B, T, W> {
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

impl<B: MutRB<T>, T, const W: bool> ConsIter<B, T, W> {
    prod_alive!();
    work_alive!();

    pub(crate) fn new(value: BufRef<B>) -> Self {
        Self {
            index: 0,
            cached_avail: 0,

            buf_len: value.inner_len(),
            buffer: value,

            _phantom: PhantomData
        }
    }

    /// Resets the index of the iterator. I.e., moves the iterator to the location occupied by its successor.
    #[inline]
    pub fn reset_index(&mut self) {
        let new_idx = self.succ_index();
        self.index = new_idx;
        self.set_index(new_idx);
    }

    /// Returns a reference to an element.
    /// <div class="warning">
    ///
    /// Being this a reference, [`Self::advance()`] has to be called when done with the data
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn peek_ref(&mut self) -> Option<&T> {
        self.next_ref()
    }

    /// Returns a tuple of slice references, the sum of which with len equal to `count`.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the data
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn peek_slice(&mut self, count: usize) -> Option<(&[T], &[T])> {
        self.next_chunk(count)
    }

    /// Returns a tuple of slice references, the sum of which with len equal to available data.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the data
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn peek_available(&mut self) -> Option<(&[T], &[T])> {
        let avail = self.available();
        self.peek_slice(avail)
    }

    /// Tries to pop an element, moving it.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
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
            dst[.. h.len()].copy_from_slice(h);
            dst[h.len() ..].copy_from_slice(t);

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
            dst[.. h.len()].clone_from_slice(h);
            dst[h.len() ..].clone_from_slice(t);

            unsafe { self.advance(dst.len()) };
            Some(())
        } else { None }
    }
}
