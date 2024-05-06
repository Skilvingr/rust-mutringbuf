use core::mem::transmute;
use core::slice;

use crate::iterators::{cons_alive, private_impl, public_impl, work_alive};
use crate::iterators::iterator_trait::{MRBIterator, PrivateMRBIterator};
use crate::ring_buffer::storage::storage_trait::Storage;
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, IterManager, MutRB};
use crate::ring_buffer::wrappers::buf_ref::BufRef;
use crate::ring_buffer::wrappers::unsafe_sync_cell::UnsafeSyncCell;

#[doc = r##"
Iterator used to push data into the buffer.

When working with types which implement both
[`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html) and
[`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html) traits, `copy` methods should be
preferred over `clone` methods.
"##]

pub struct ProdIter<B: MutRB> {
    index: usize,
    buf_len: usize,
    buffer: BufRef<B>,

    cached_avail: usize,
}

unsafe impl<B: ConcurrentRB + MutRB<Item = T>, T> Send for ProdIter<B> {}

impl<B: MutRB + IterManager> Drop for ProdIter<B> {
    fn drop(&mut self) {
        self.buffer.set_prod_alive(false);
    }
}

impl<B: MutRB<Item = T>, T> PrivateMRBIterator<T> for ProdIter<B> {
    #[inline]
    fn set_atomic_index(&self, index: usize) {
        self.buffer.set_prod_index(index);
    }

    #[inline]
    fn succ_index(&self) -> usize {
        self.buffer.cons_index()
    }

    private_impl!();
}

impl<B: MutRB<Item = T>, T> MRBIterator<T> for ProdIter<B> {
    #[inline]
    fn available(&mut self) -> usize {
        let succ_idx = self.succ_index();

        self.cached_avail = match self.index < succ_idx {
            true => succ_idx - self.index - 1,
            false => self.buf_len - self.index + succ_idx - 1
        };

        self.cached_avail
    }

    public_impl!();
}

impl<B: MutRB<Item = T>, T> ProdIter<B> {
    work_alive!();
    cons_alive!();

    pub(crate) fn new(value: BufRef<B>) -> Self {
        Self {
            index: 0,
            buf_len: value.inner_len(),
            buffer: value,
            cached_avail: 0
        }
    }

    /// Tries to push a new item by moving or copying it.
    ///
    /// Returns:
    /// * `Err(value)`, if the buffer is full;
    /// * `Ok(())`, otherwise.
    #[inline]
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if let Some(binding) = self.next_ref_mut_init() {
            unsafe { binding.write(value) };
            unsafe { self.advance(1) };
            Ok(())
        } else {
            Err(value)
        }
    }

    /// Tries to push a slice of items by copying the elements.
    /// The elements must implement [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html) trait.
    ///
    /// Returns:
    /// * `None`, if the buffer is full;
    /// * `Some(())`, otherwise.
    #[inline]
    pub fn push_slice(&mut self, slice: &[T]) -> Option<()>
        where T: Copy
    {
        let count = slice.len();

        if let Some((binding_h, binding_t)) = self.next_chunk_mut(count) {

            let mid = binding_h.len();
            if mid == count {
                binding_h.copy_from_slice(slice);
            } else {
                binding_h.copy_from_slice(&slice[.. mid]);
                binding_t.copy_from_slice(&slice[mid ..]);
            }

            unsafe { self.advance(count) };
            Some(())
        } else {
            None
        }
    }

    /// Tries to push a slice of items by cloning the elements.
    /// The elements must implement [`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html) trait.
    ///
    /// Returns:
    /// * `None`, if the buffer is full;
    /// * `Some(())`, otherwise.
    #[inline]
    pub fn push_slice_clone(&mut self, slice: &[T]) -> Option<()>
        where T: Clone
    {
        let count = slice.len();

        if let Some((binding_h, binding_t)) = self.next_chunk_mut(count) {

            let mid = binding_h.len();
            if mid == count {
                binding_h.clone_from_slice(slice);
            } else {
                binding_h.clone_from_slice(&slice[.. mid]);
                binding_t.clone_from_slice(&slice[mid ..]);
            }

            unsafe { self.advance(count) };
            Some(())
        } else {
            None
        }
    }

    /// If available, returns a mutable reference to the next item.
    /// This reference can be used to write data into an *initialised* item.
    ///
    /// Items can be initialised by calling [`Self::get_next_item_mut_init`] or by creating a buffer
    /// using `default` constructor. E.g.: `ConcurrentHeapRB::default` or `LocalStackRB::default`.
    ///
    /// For uninitialised items, use [`Self::get_next_item_mut_init`], instead.
    ///
    /// <div class="warning">
    ///
    /// Being this a reference, [`Self::advance`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    ///
    /// # Safety
    /// The retrieved item must be initialised! For more info, refer to [`MaybeUninit::assume_init_mut`](https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#method.assume_init_mut).
    pub unsafe fn get_next_item_mut(&mut self) -> Option<&mut T> {
        self.next_ref_mut()
    }

    /// If available, returns a mutable pointer to the next item.
    /// This pointer can be used to write data into the item, even if this is not already initialised.
    /// It is important to note that reading from this pointer or turning it into a reference is still
    /// undefined behavior, unless the item is initialized.
    ///
    /// For more info, refer to [`MaybeUninit::as_mut_ptr`](https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#method.as_mut_ptr).
    /// <div class="warning">
    ///
    /// Being this a pointer, [`Self::advance`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    pub fn get_next_item_mut_init(&mut self) -> Option<*mut T> {
        self.next_ref_mut_init()
    }
}