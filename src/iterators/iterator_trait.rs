use core::mem::transmute;
use core::slice;
use crate::iterators::sync_iterators::detached::Detached;
use crate::{MutRB, Storage, UnsafeSyncCell};
use crate::ring_buffer::wrappers::buf_ref::BufRef;
use crate::ring_buffer::variants::ring_buffer_trait::{IterManager, StorageManager};

/// Returned by slice-specialised functions.
/// # Fields:
/// - 1: head
/// - 2: tail
pub type WorkableSlice<'a, T> = (&'a mut [T], &'a mut [T]);


/// Trait implemented by iterators.
#[allow(private_bounds)]
pub trait MRBIterator: PrivateMRBIterator<Self::Item> {
    type Item;

    /// Detaches the iterator yielding a [`Detached`].
    #[inline]
    fn detach(self) -> Detached<Self> where Self: Sized {
        Detached::from_iter(self)
    }
    
    /// Advances the iterator by `count`.
    ///
    /// # Safety
    /// An iterator should never overstep its successor, so it must always be: `count` <= [`MRBIterator::available()`]!
    #[inline]
    unsafe fn advance(&mut self, count: usize) {
        self._advance(count);
    }

    /// Returns the number of items available for an iterator.
    #[inline]
    fn available(&mut self) -> usize {
        self._available()
    }

    /// Waits, blocking the thread in a loop, until there are at least `count` available items.
    fn wait_for(&mut self, count: usize) {
        while self.available() < count {}
    }

    /// Returns the index of the iterator.
    #[inline]
    fn index(&self) -> usize {
        self._index()
    }

    /// Returns the length of the buffer.
    #[inline]
    fn buf_len(&self) -> usize {
        self.buffer().inner_len()
    }
    
    /// Returns `true` if the producer iterator is still alive, `false` if it has been dropped.
    fn is_prod_alive(&self) -> bool {
        self.buffer().prod_alive()
    }
    /// Returns `true` if the worker iterator is still alive, `false` if it has been dropped.
    fn is_work_alive(&self) -> bool {
        self.buffer().work_alive()
    }
    /// Returns `true` if the consumer iterator is still alive, `false` if it has been dropped.
    fn is_cons_alive(&self) -> bool {
        self.buffer().cons_alive()
    }
    /// Returns the index of the producer.
    #[inline(always)]
    fn prod_index(&self) -> usize {
        self.buffer().prod_index()
    }
    /// Returns the index of the worker.
    #[inline(always)]
    fn work_index(&self) -> usize {
        self.buffer().work_index()
    }
    /// Returns the index of the consumer.
    #[inline(always)]
    fn cons_index(&self) -> usize {
        self.buffer().cons_index()
    }
    
    /// Returns a mutable references to the current value.
    ///
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    fn get_workable<'a>(&mut self) -> Option<&'a mut Self::Item> {
        self.next_ref_mut()
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to `count`.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    fn get_workable_slice_exact<'a>(&mut self, count: usize) -> Option<WorkableSlice<'a, <Self as MRBIterator>::Item>> {
        self.next_chunk_mut(count)
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to [`Self::available()`].
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    fn get_workable_slice_avail<'a>(&mut self) -> Option<WorkableSlice<'a, <Self as MRBIterator>::Item>> {
        match self.available() {
            0 => None,
            avail => self.get_workable_slice_exact(avail)
        }
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to the
    /// higher multiple of `rhs`.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    fn get_workable_slice_multiple_of<'a>(&mut self, rhs: usize) -> Option<WorkableSlice<'a, <Self as MRBIterator>::Item>> {
        let avail = self.available();

        unsafe {
            match avail.unchecked_sub(avail % rhs) {
                0 => None,
                avail => self.get_workable_slice_exact(avail)
            }
        }
    }
}

pub(crate) trait PrivateMRBIterator<T> {
    fn buffer(&self) -> &BufRef<'_, impl MutRB<Item = T>>;
    fn _available(&mut self) -> usize;
    fn cached_avail(&self) -> usize;
    fn set_cached_avail(&mut self, avail: usize);
    fn _index(&self) -> usize;
    fn set_local_index(&mut self, index: usize);
    /// Sets the global index of this iterator.
    fn set_atomic_index(&self, index: usize);

    /// Returns the global index of successor.
    fn succ_index(&self) -> usize;

    #[inline]
    unsafe fn _advance(&mut self, count: usize) {
        self.advance_local(count);

        self.set_atomic_index(self._index());
    }
    
    #[inline]
    unsafe fn advance_local(&mut self, count: usize) {
        self.set_local_index(self._index().unchecked_add(count));

        if self._index() >= self.buffer().inner_len() {
            self.set_local_index(self._index().unchecked_sub(self.buffer().inner_len()));
        }

        self.set_cached_avail(self.cached_avail().saturating_sub(count));
    }
    
    /// Checks whether the current index can be returned
    #[inline]
    fn check(&mut self, count: usize) -> bool {
        self.cached_avail() >= count || self._available() >= count
    }

    /// Returns Some(current element), if `check()` returns `true`, else None
    #[inline]
    fn next(&mut self) -> Option<T> {
        self.check(1).then(|| unsafe {
            let ret = self.buffer().inner()[self._index()].take_inner();

            self._advance(1);

            ret
        })
    }

    /// Returns Some(current element), if `check()` returns `true`, else None. The value is duplicated.
    #[inline]
    fn next_duplicate(&mut self) -> Option<T> {
        self.check(1).then(|| unsafe {
            let ret = self.buffer().inner()[self._index()].inner_duplicate();

            self._advance(1);

            ret
        })
    }

    /// Returns Some(&UnsafeSyncCell<current element>), if `check()` returns `true`, else None
    #[inline]
    fn next_ref<'a>(&mut self) -> Option<&'a T> {
        unsafe { self.check(1).then(|| self.buffer().inner()[self._index()].inner_ref()) }
    }

    /// Returns Some(&UnsafeSyncCell<current element>), if `check()` returns `true`, else None
    #[inline]
    fn next_ref_mut<'a>(&mut self) -> Option<&'a mut T> {
        unsafe { self.check(1).then(|| self.buffer().inner()[self._index()].inner_ref_mut()) }
    }

    /// As next_ref_mut, but can be used for initialisation of inner MaybeUninit.
    #[inline]
    fn next_ref_mut_init(&mut self) -> Option<*mut T> {
        self.check(1).then(|| self.buffer().inner()[self._index()].as_mut_ptr())
    }

    #[inline]
    fn next_chunk<'a>(&mut self, count: usize) -> Option<(&'a [T], &'a [T])> {
        self.check(count).then(|| {

            let len = self.buffer().inner_len();
            
            unsafe {
                let ptr = self.buffer().inner().as_ptr();

                if self._index() + count >= len {
                    (
                        transmute::<&[UnsafeSyncCell<T>], &[T]>(
                            slice::from_raw_parts(ptr.add(self._index()), len.unchecked_sub(self._index()))
                        ),
                        transmute::<&[UnsafeSyncCell<T>], &[T]>(
                            slice::from_raw_parts(ptr, self._index().unchecked_add(count).unchecked_sub(len))
                        )
                    )
                } else {
                    (
                        transmute::<&[UnsafeSyncCell<T>], &[T]>(
                            slice::from_raw_parts(ptr.add(self._index()), count)
                        ),
                        &mut [] as &[T]
                    )
                }
            }
        })
    }

    #[inline]
    fn next_chunk_mut<'a>(&mut self, count: usize) -> Option<(&'a mut [T], &'a mut [T])> {
        self.check(count).then(|| {

            let len = self.buffer().inner_len();
            
            unsafe {
                let ptr = self.buffer().inner_mut().as_mut_ptr();

                if self._index() + count >= len {
                    (
                        transmute::<&mut [UnsafeSyncCell<T>], &mut [T]>(
                            slice::from_raw_parts_mut(ptr.add(self._index()), len.unchecked_sub(self._index()))
                        ),
                        transmute::<&mut [UnsafeSyncCell<T>], &mut [T]>(
                            slice::from_raw_parts_mut(ptr, self._index().unchecked_add(count).unchecked_sub(len))
                        )
                    )
                } else {
                    (
                        transmute::<&mut [UnsafeSyncCell<T>], &mut [T]>(
                            slice::from_raw_parts_mut(ptr.add(self._index()), count)
                        ),
                        &mut [] as &mut [T]
                    )
                }
            }
        })
    }
}

pub(crate) mod iter_macros {
    macro_rules! private_impl { () => (

        #[inline]
        fn buffer(&self) -> &BufRef<'_, impl MutRB<Item = T>> {
            &self.buffer
        }
        
        #[inline]
        fn _index(&self) -> usize {
            self.index
        }
        #[inline]
        fn set_local_index(&mut self, index: usize) {
            self.index = index;
        }
        
        #[inline]
        fn cached_avail(&self) -> usize {
            self.cached_avail
        }
        #[inline]
        fn set_cached_avail(&mut self, avail: usize) {
            self.cached_avail = avail;
        }
    )}

    pub(crate) use private_impl;
}