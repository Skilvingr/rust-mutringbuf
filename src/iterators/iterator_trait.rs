/// Trait implemented by iterators.
pub trait MRBIterator<T> {
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

    /// Returns the length of the buffer.
    fn buf_len(&self) -> usize;
}

pub(crate) trait PrivateMRBIterator<T> {
    unsafe fn advance_local(&mut self, count: usize);
    
    /// Sets the global index of this iterator.
    fn set_atomic_index(&self, index: usize);

    /// Returns the global index of successor.
    fn succ_index(&self) -> usize;

    /// Checks whether the current index can be returned
    fn check(&mut self, count: usize) -> bool;

    /// Returns Some(current element), if `check()` returns `true`, else None
    fn next(&mut self) -> Option<T>;

    /// Returns Some(current element), if `check()` returns `true`, else None. The value is duplicated.
    fn next_duplicate(&mut self) -> Option<T>;

    /// Returns Some(&UnsafeSyncCell<current element>), if `check()` returns `true`, else None
    fn next_ref<'a>(&mut self) -> Option<&'a T>;

    /// Returns Some(&UnsafeSyncCell<current element>), if `check()` returns `true`, else None
    fn next_ref_mut<'a>(&mut self) -> Option<&'a mut T>;

    /// As next_ref_mut, but can be used for initialisation of inner MaybeUninit.
    fn next_ref_mut_init(&mut self) -> Option<*mut T>;

    fn next_chunk<'a>(&mut self, count: usize) -> Option<(&'a [T], &'a [T])>;

    fn next_chunk_mut<'a>(&mut self, count: usize) -> Option<(&'a mut [T], &'a mut [T])>;
}

pub(crate) mod iter_macros {
    macro_rules! prod_alive { () => (
        /// Returns `true` if the producer iterator is still alive, `false` if it has been dropped.
        #[inline]
        pub fn is_prod_alive(&self) -> bool {
            self.buffer.prod_alive()
        }
    )}
    macro_rules! work_alive { () => (
        /// Returns `true` if the worker iterator is still alive, `false` if it has been dropped.
        ///
        /// Note: when the buffer is used in non-mutable mode this will always return `false`.
        #[inline]
        pub fn is_work_alive(&self) -> bool {
            self.buffer.work_alive()
        }
    )}
    macro_rules! cons_alive { () => (
        /// Returns `true` if the consumer iterator is still alive, `false` if it has been dropped.
        #[inline]
        pub fn is_cons_alive(&self) -> bool {
            self.buffer.cons_alive()
        }
    )}

    macro_rules! prod_index { () => (
        /// Returns the index of the producer.
        #[inline]
        pub fn prod_index(&self) -> usize {
            self.buffer.prod_index()
        }
    )}
    macro_rules! work_index { () => (
        /// Returns the index of the worker.
        /// Note: when the buffer is used in non-mutable mode this will always return `0`.
        #[inline]
        pub fn work_index(&self) -> usize {
            self.buffer.work_index()
        }
    )}
    macro_rules! cons_index { () => (
        /// Returns the index of the consumer.
        #[inline]
        pub fn cons_index(&self) -> usize {
            self.buffer.cons_index()
        }
    )}

    macro_rules! public_impl { () => (
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
        fn buf_len(&self) -> usize {
            self.buf_len.get()
        }
    )}

    macro_rules! private_impl { () => (
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

    pub(crate) use { public_impl, private_impl, prod_alive, work_alive, cons_alive, prod_index, work_index, cons_index };
}