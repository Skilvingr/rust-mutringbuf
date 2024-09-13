use crate::iterators::sync_iterators::detached::Detached;

/// Returned by slice-specialised functions.
/// # Fields:
/// - 1: head
/// - 2: tail
pub type WorkableSlice<'a, T> = (&'a mut [T], &'a mut [T]);


/// Trait implemented by iterators.
#[allow(private_bounds)]
pub trait MRBIterator: PrivateMRBIterator<PItem = Self::Item> {
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
    unsafe fn advance(&mut self, count: usize);

    /// Returns the number of items available for an iterator.
    fn available(&mut self) -> usize;

    /// Waits, blocking the thread in a loop, until there are at least `count` available items.
    fn wait_for(&mut self, count: usize) {
        while self.available() < count {}
    }

    /// Returns the index of the iterator.
    fn index(&self) -> usize;

    /// Sets the local index.
    fn set_index(&mut self, index: usize);

    /// Returns the length of the buffer.
    fn buf_len(&self) -> usize;

    /// Returns `true` if the producer iterator is still alive, `false` if it has been dropped.
    fn is_prod_alive(&self) -> bool;
    /// Returns `true` if the worker iterator is still alive, `false` if it has been dropped.
    fn is_work_alive(&self) -> bool;
    /// Returns `true` if the consumer iterator is still alive, `false` if it has been dropped.
    fn is_cons_alive(&self) -> bool;
    /// Returns the index of the producer.
    fn prod_index(&self) -> usize;
    /// Returns the index of the worker.
    fn work_index(&self) -> usize;
    /// Returns the index of the consumer.
    fn cons_index(&self) -> usize;
    
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

        match avail - avail % rhs {
            0 => None,
            avail => self.get_workable_slice_exact(avail)
        }
    }
}

pub(crate) trait PrivateMRBIterator {
    type PItem;
    
    fn cached_avail(&mut self) -> usize;
    fn set_cached_avail(&mut self, avail: usize);
    unsafe fn set_local_index(&mut self, index: usize);
    
    unsafe fn advance_local(&mut self, count: usize);
    
    /// Sets the global index of this iterator.
    fn set_atomic_index(&self, index: usize);

    /// Returns the global index of successor.
    fn succ_index(&self) -> usize;

    /// Checks whether the current index can be returned
    fn check(&mut self, count: usize) -> bool;

    /// Returns Some(current element), if `check()` returns `true`, else None
    fn next(&mut self) -> Option<Self::PItem>;

    /// Returns Some(current element), if `check()` returns `true`, else None. The value is duplicated.
    fn next_duplicate(&mut self) -> Option<Self::PItem>;

    /// Returns Some(&UnsafeSyncCell<current element>), if `check()` returns `true`, else None
    fn next_ref<'a>(&mut self) -> Option<&'a Self::PItem>;

    /// Returns Some(&UnsafeSyncCell<current element>), if `check()` returns `true`, else None
    fn next_ref_mut<'a>(&mut self) -> Option<&'a mut Self::PItem>;

    /// As next_ref_mut, but can be used for initialisation of inner MaybeUninit.
    fn next_ref_mut_init(&mut self) -> Option<*mut Self::PItem>;

    fn next_chunk<'a>(&mut self, count: usize) -> Option<(&'a [Self::PItem], &'a [Self::PItem])>;

    fn next_chunk_mut<'a>(&mut self, count: usize) -> Option<(&'a mut [Self::PItem], &'a mut [Self::PItem])>;
}

pub(crate) mod iter_macros {
    macro_rules! public_impl { () => (
        #[inline]
        fn is_prod_alive(&self) -> bool {
            self.buffer.prod_alive()
        }
    
        #[inline]
        fn is_work_alive(&self) -> bool {
            self.buffer.work_alive()
        }
    
        #[inline]
        fn is_cons_alive(&self) -> bool {
            self.buffer.cons_alive()
        }
    
        #[inline]
        fn prod_index(&self) -> usize {
            self.buffer.prod_index()
        }
    
        #[inline]
        fn work_index(&self) -> usize {
            self.buffer.work_index()
        }
    
        #[inline]
        fn cons_index(&self) -> usize {
            self.buffer.cons_index()
        }
        
        #[inline]
        unsafe fn advance(&mut self, count: usize) {
            self.advance_local(count);

            self.set_atomic_index(self.index);
        }

        #[inline(always)]
        fn index(&self) -> usize {
            self.index
        }
        
        #[inline(always)]
        fn set_index(&mut self, index: usize) {
            self.index = index;
        }

        #[inline(always)]
        fn buf_len(&self) -> usize {
            self.buf_len.get()
        }
    )}

    macro_rules! private_impl { () => (
        #[inline]
        fn cached_avail(&mut self) -> usize {
            self.cached_avail
        }
    
        #[inline]
        fn set_cached_avail(&mut self, avail: usize) {
            self.cached_avail = avail;
        }
    
        #[inline]
        unsafe fn set_local_index(&mut self, index: usize) {
            self.index = index;
        }
            
        #[inline]
        unsafe fn advance_local(&mut self, count: usize) {
            self.index += count;
            
            if self.index >= self.buf_len.get() {
                self.index -= self.buf_len.get();
            }
            
            self.cached_avail = self.cached_avail.saturating_sub(count);
        }
        
        #[inline(always)]
        fn check(&mut self, count: usize) -> bool {
            self.cached_avail >= count || self.available() >= count
        }

        #[inline]
        fn next(&mut self) -> Option<T> {
            self.check(1).then(|| {
                let ret = unsafe { self.buffer.inner()[self.index].take_inner() };

                unsafe { self.advance(1) };

                ret
            })
        }
        
        #[inline]
        fn next_duplicate(&mut self) -> Option<T> {
            self.check(1).then(|| {
                let ret = unsafe { self.buffer.inner()[self.index].inner_duplicate() };

                unsafe { self.advance(1) };

                ret
            })
        }

        #[inline]
        fn next_ref<'a>(&mut self) -> Option<&'a T> {
            unsafe { self.check(1).then(|| self.buffer.inner()[self.index].inner_ref()) }
        }

        #[inline]
        fn next_ref_mut<'a>(&mut self) -> Option<&'a mut T> {
            unsafe { self.check(1).then(|| self.buffer.inner()[self.index].inner_ref_mut()) }
        }

        #[inline]
        fn next_ref_mut_init(&mut self) -> Option<*mut T> {
            self.check(1).then(|| self.buffer.inner()[self.index].as_mut_ptr())
        }

        #[inline]
        fn next_chunk<'a>(&mut self, count: usize) -> Option<(&'a [T], &'a [T])> {
            self.check(count).then(|| {
                
                unsafe {
                    let ptr = self.buffer.inner().as_ptr();
                    
                    if self.index + count >= self.buf_len.get() {
                        (
                            transmute::<&[UnsafeSyncCell<T>], &[T]>(
                                slice::from_raw_parts(ptr.add(self.index), self.buf_len.get() - self.index)
                            ),
                            transmute::<&[UnsafeSyncCell<T>], &[T]>(
                                slice::from_raw_parts(ptr, self.index + count - self.buf_len.get())
                            )
                        )
                    } else {
                        (
                            transmute::<&[UnsafeSyncCell<T>], &[T]>(
                                slice::from_raw_parts(ptr.add(self.index), count)
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
                
                unsafe {
                    let ptr = self.buffer.inner_mut().as_mut_ptr();
                    
                    if self.index + count >= self.buf_len.get() {
                        (
                            transmute::<&mut [UnsafeSyncCell<T>], &mut [T]>(
                                slice::from_raw_parts_mut(ptr.add(self.index), self.buf_len.get() - self.index)
                            ),
                            transmute::<&mut [UnsafeSyncCell<T>], &mut [T]>(
                                slice::from_raw_parts_mut(ptr, self.index + count - self.buf_len.get())
                            )
                        )
                    } else {
                        (
                            transmute::<&mut [UnsafeSyncCell<T>], &mut [T]>(
                                slice::from_raw_parts_mut(ptr.add(self.index), count)
                            ),
                            &mut [] as &mut [T]
                        )
                    }
                }
            })
        }
    )}

    pub(crate) use { public_impl, private_impl };
}