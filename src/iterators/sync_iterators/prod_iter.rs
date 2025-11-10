#[cfg(doc)]
use {crate::iterators::ConsIter, crate::iterators::Detached, core::mem::MaybeUninit};

use crate::iterators::iterator_trait::{MRBIterator, MutableSlice, PrivateMRBIterator};
use crate::iterators::{copy_from_slice_unchecked, private_impl};
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, IterManager, MutRB};
use crate::ring_buffer::wrappers::buf_ref::BufRef;
use crate::ring_buffer::wrappers::unsafe_sync_cell::UnsafeSyncCell;

#[doc = r##"
Iterator used to push data into the buffer.

When working with types which implement both
[`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html) and
[`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html) traits, `copy` methods should be
preferred over `clone` methods.

# TL;DR about uninitialised memory
If you created the buffer with either `default` or `from` methods and you are *not* going to use [`ConsIter::pop`],
then it is safe to use normal methods from this struct.

If you either created the buffer with `new_zeroed` or are going to move items out of the buffer (e.g. using [`ConsIter::pop`]),
then you must pay attention to what you do and ensure that all the locations cleared by `pop` will be re-filled with `*_init` methods.
After that you can use normal methods again.
Read below to know why and how.

It would be a good idea to do a check with [miri](https://github.com/rust-lang/miri), which is able to
tell if and when something bad has happened.

# A note about how this buffer is made:
Every element in this buffer is wrapped in an [`UnsafeSyncCell`], which in the end is a [`MaybeUninit`].
`MaybeUninit` (read the official docs if you want to know more) is a way to deal with possibly uninitialised data.

When one creates a buffer from this crate, they may choose to build it either with `default` and `from` methods,
or with `new_zeroed` ones.

When using the former, an initialised buffer is created, so there are no problems concerning uninitialised memory.
With the latter methods, a zeroed buffer is rather created.
To write data into an uninitialised (or zeroed) block of memory, one has to use [`write`](https://doc.rust-lang.org/std/primitive.pointer.html#method.write)
method, which overwrites a memory location, without reading or dropping the old value.

Remember that a zeroed location must *never* be read or dropped. Doing so would cause UB!

On the other hand, using `write` on an initialised block doesn't drop the old value, causing then a memory leak.

For each of the methods in this struct, there exists a `*_init` one (e.g. [`Self::push`] and [`Self::push_init`]).
Normal methods are faster than `*_init` ones, and should be preferred over these when dealing with *surely*
initialised memory.

On the other hand, `*_init methods` always perform a check over the memory they are going to write and choose the proper way to
deal it, even dropping the old value, if there is the need. So they are safe to use upon a possibly uninitialised block.
"##]
pub struct ProdIter<'buf, B: MutRB> {
    index: usize,
    cached_avail: usize,
    buffer: BufRef<'buf, B>,
}

unsafe impl<B: ConcurrentRB + MutRB<Item = T>, T> Send for ProdIter<'_, B> {}

impl<B: MutRB + IterManager> Drop for ProdIter<'_, B> {
    fn drop(&mut self) {
        self.buffer.drop_iter();
    }
}

impl<B: MutRB<Item = T>, T> PrivateMRBIterator<T> for ProdIter<'_, B> {
    #[inline]
    fn _available(&mut self) -> usize {
        let succ_idx = self.succ_index();

        unsafe {
            self.cached_avail = match self.index < succ_idx {
                true => succ_idx.unchecked_sub(self.index).unchecked_sub(1),
                false => self
                    .buf_len()
                    .unchecked_sub(self.index)
                    .unchecked_add(succ_idx)
                    .unchecked_sub(1),
            };
        }

        self.cached_avail
    }

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

impl<B: MutRB<Item = T>, T> MRBIterator for ProdIter<'_, B> {
    type Item = T;
}

impl<'buf, B: MutRB<Item = T>, T> ProdIter<'buf, B> {
    pub(crate) fn new(value: BufRef<'buf, B>) -> Self {
        Self {
            index: 0,
            buffer: value,
            cached_avail: 0,
        }
    }

    #[inline]
    fn _push(&mut self, value: T, f: fn(*mut T, T)) -> Result<(), T> {
        if let Some(binding) = self.next_ref_mut_init() {
            f(binding, value);
            unsafe { self.advance(1) };
            Ok(())
        } else {
            Err(value)
        }
    }

    /// Tries to push a new item by moving or copying it.
    ///
    /// This method must *not* be used to push items after a [`ConsIter::pop`].
    /// In this case, [`Self::push_init`] has to be used, instead.
    /// For more info, refer to the main documentation above.
    ///
    /// Returns:
    /// * `Err(value)`, if the buffer is full;
    /// * `Ok(())`, otherwise.
    #[inline]
    pub fn push(&mut self, value: T) -> Result<(), T> {
        fn f<T>(binding: *mut T, value: T) {
            unsafe {
                *binding = value;
            }
        }

        self._push(value, f)
    }

    /// Same as [`Self::push_slice`], but can be used when dealing with possibly uninitialised
    /// locations within the buffer, e.g. after a [`ConsIter::pop`].
    ///
    /// Returns:
    /// * `Err(value)`, if the buffer is full;
    /// * `Ok(())`, otherwise.
    #[inline]
    pub fn push_init(&mut self, value: T) -> Result<(), T> {
        fn f<T>(binding: *mut T, value: T) {
            unsafe {
                if UnsafeSyncCell::check_zeroed(binding) {
                    binding.write(value);
                } else {
                    *binding = value;
                }
            }
        }

        self._push(value, f)
    }

    #[cfg(feature = "vmem")]
    #[inline]
    fn _push_slice(&mut self, slice: &[T], f: fn(&mut [T], &[T])) -> Option<()> {
        let count = slice.len();

        if let Some(binding) = self.next_chunk_mut(count) {
            f(binding, slice);

            unsafe { self.advance(count) };
            Some(())
        } else {
            None
        }
    }

    #[cfg(not(feature = "vmem"))]
    #[inline]
    fn _push_slice(&mut self, slice: &[T], f: fn(&mut [T], &[T])) -> Option<()> {
        let count = slice.len();

        if let Some((binding_h, binding_t)) = self.next_chunk_mut(count) {
            let mid = binding_h.len();
            if mid == slice.len() {
                f(binding_h, slice);
            } else {
                unsafe {
                    f(binding_h, slice.get_unchecked(..mid));
                    f(binding_t, slice.get_unchecked(mid..));
                }
            }

            unsafe { self.advance(count) };
            Some(())
        } else {
            None
        }
    }

    /// Tries to push a slice of items by copying the elements.
    /// The elements must implement [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html) trait.
    ///
    /// This method must *not* be used to push items after a [`ConsIter::pop`].
    /// In this case, [`Self::push_slice_init`] has to be used, instead.
    /// For more info, refer to the main documentation above.
    ///
    /// Returns:
    /// * `None`, if the buffer is full;
    /// * `Some(())`, otherwise.
    #[inline]
    pub fn push_slice(&mut self, slice: &[T]) -> Option<()>
    where
        T: Copy,
    {
        #[inline]
        fn f<T: Copy>(binding: &mut [T], slice: &[T]) {
            copy_from_slice_unchecked(slice, binding);
        }

        self._push_slice(slice, f)
    }

    /// Same as [`Self::push_slice`], but can be used when dealing with possibly uninitialised
    /// locations within the buffer, e.g. after a [`ConsIter::pop`].
    ///
    /// Returns:
    /// * `None`, if the buffer is full;
    /// * `Some(())`, otherwise.
    #[inline]
    pub fn push_slice_init(&mut self, slice: &[T]) -> Option<()>
    where
        T: Copy,
    {
        #[inline]
        fn f<T: Copy>(binding_h: &mut [T], slice: &[T]) {
            for (x, y) in binding_h.iter_mut().zip(slice) {
                if UnsafeSyncCell::check_zeroed(x as *mut T) {
                    unsafe {
                        (x as *mut T).write(*y);
                    }
                } else {
                    *x = *y;
                }
            }
        }

        self._push_slice(slice, f)
    }

    /// Tries to push a slice of items by cloning the elements.
    /// The elements must implement [`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html) trait.
    ///
    /// This method must *not* be used to push items after a [`ConsIter::pop`].
    /// In this case, [`Self::push_slice_clone_init`] has to be used, instead.
    /// For more info, refer to the main documentation above.
    ///
    /// Returns:
    /// * `None`, if the buffer is full;
    /// * `Some(())`, otherwise.
    #[inline]
    pub fn push_slice_clone(&mut self, slice: &[T]) -> Option<()>
    where
        T: Clone,
    {
        #[inline]
        fn f<T: Clone>(binding_h: &mut [T], slice: &[T]) {
            binding_h.clone_from_slice(slice);
        }

        self._push_slice(slice, f)
    }

    /// Same as [`Self::push_slice_clone`], but can be used when dealing with possibly uninitialised
    /// locations within the buffer, e.g. after a [`ConsIter::pop`].
    ///
    /// Returns:
    /// * `None`, if the buffer is full;
    /// * `Some(())`, otherwise.
    #[inline]
    pub fn push_slice_clone_init(&mut self, slice: &[T]) -> Option<()>
    where
        T: Clone,
    {
        #[inline]
        fn f<T: Clone>(binding_h: &mut [T], slice: &[T]) {
            for (x, y) in binding_h.iter_mut().zip(slice) {
                if UnsafeSyncCell::check_zeroed(x as *mut T) {
                    unsafe {
                        (x as *mut T).write(y.clone());
                    }
                } else {
                    x.clone_from(y);
                }
            }
        }

        self._push_slice(slice, f)
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
    pub unsafe fn get_next_item_mut<'a>(&mut self) -> Option<&'a mut T> {
        self.next_ref_mut()
    }

    /// If available, returns a mutable pointer to the next item.
    /// This pointer can be used to write data into the item, even if this is not already initialised.
    /// It is important to note that reading from this pointer or turning it into a reference is still
    /// undefined behavior, unless the item is initialized.
    ///
    /// If the memory pointed by this pointer is already initialised, it is possible to write into it
    /// with a simple:
    /// ```ignore
    /// *ptr = value;
    /// ```
    /// Doing so, the old value will be automatically dropped and no leak will be created.
    ///
    /// If the memory is not initialised, the write must be done with:
    /// ```ignore
    /// ptr.write(value);
    /// ```
    /// The reason is that `write` does not drop the old value, which is good, because dropping an
    /// uninitialised value is UB!
    ///
    /// One should be able to test whether a piece of memory is initialised with [`UnsafeSyncCell::check_zeroed`].
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

    /// If available, returns two mutable slices with a total count equal to `count`.
    /// These references can be used to write data into *initialised* items.
    ///
    /// Items can be initialised (one by one) by calling [`Self::get_next_item_mut_init`] or by creating a buffer
    /// using `default` constructor. E.g.: `ConcurrentHeapRB::default` or `LocalStackRB::default`.
    ///
    /// <div class="warning">
    ///
    /// Being these reference, [`Self::advance`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    ///
    /// # Safety
    /// The retrieved items must be initialised! For more info, refer to [`MaybeUninit::assume_init_mut`](https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#method.assume_init_mut).
    pub unsafe fn get_next_slices_mut<'a>(&mut self, count: usize) -> Option<MutableSlice<'a, T>> {
        self.next_chunk_mut(count)
    }
}

pub mod test {
    #[test]
    fn cached_avail() {
        use super::*;
        use crate::{ConcurrentHeapRB, HeapSplit};

        const BUFFER_SIZE: usize = 4095;

        let buf = ConcurrentHeapRB::<u32>::default(BUFFER_SIZE + 1);
        let (mut prod, mut cons) = buf.split();

        assert_eq!(prod.cached_avail, 0);

        prod.check(1);

        assert_eq!(prod.cached_avail, BUFFER_SIZE);

        unsafe {
            prod.advance(2);
        }

        assert_eq!(prod.cached_avail, BUFFER_SIZE - 2);

        unsafe {
            cons.advance(1);
        }

        assert_eq!(prod.cached_avail, BUFFER_SIZE - 2);

        prod.check(10);

        assert_eq!(prod.cached_avail, BUFFER_SIZE - 2);
    }
}
