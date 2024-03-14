use core::mem::transmute;
use core::slice;

use crate::iterators::{cons_alive, private_impl, public_impl, work_alive};
use crate::iterators::iterable::{Iterable, PrivateIterable};
use crate::ring_buffer::storage::storage_trait::Storage;
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, IterManager, StorageManager};
use crate::ring_buffer::wrappers::buf_ref::BufRef;
use crate::ring_buffer::wrappers::unsafe_sync_cell::UnsafeSyncCell;

#[doc = r##"
Iterator used to push data within the buffer.

When used to push slices, if working with types which implement both
[`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html) and
[`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html) traits, [`Self::push_slice`]
should be preferred over [`Self::push_slice_clone`].
"##]

pub struct ProdIter<
    B: IterManager + StorageManager<StoredType = T>,
    T,
> {
    index: usize,
    buf_len: usize,
    cached_avail: usize,

    buffer: BufRef<B>,
}

unsafe impl<B: ConcurrentRB + IterManager + StorageManager<StoredType = T>, T> Send for ProdIter<B, T> {}

impl<B: IterManager + StorageManager<StoredType = T>,
    T> Drop for ProdIter<B, T> {
    fn drop(&mut self) {
        self.buffer.set_prod_alive(false);
    }
}

impl<B: IterManager + StorageManager<StoredType = T>, T,> PrivateIterable<T> for ProdIter<B, T> {
    #[inline]
    fn set_index(&self, index: usize) {
        self.buffer.set_prod_index(index);
    }

    #[inline]
    fn succ_index(&self) -> usize {
        self.buffer.cons_index()
    }

    private_impl!();
}

impl<B: IterManager + StorageManager<StoredType = T>, T> Iterable<T> for ProdIter<B, T> {
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

impl<B: IterManager + StorageManager<StoredType = T>, T> ProdIter<B, T> {
    work_alive!();
    cons_alive!();

    pub(crate) fn new(value: BufRef<B>) -> Self {
        Self {
            index: 0,
            cached_avail: 0,

            buf_len: value.inner_len(),
            buffer: value,
        }
    }

    /// Tries to push a new item moving it.
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

    /// Tries to push a slice of items copying the elements.
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

    /// Tries to push a slice of items cloning the elements.
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
}