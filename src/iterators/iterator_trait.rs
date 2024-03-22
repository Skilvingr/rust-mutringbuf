pub trait MRBIterator<T> {
    /// Advances the iterator by `count`.
    ///
    /// # Safety
    /// An iterator should never overstep its successor, so it must always be: `count` <= [`MRBIterator::available()`]!
    unsafe fn advance(&mut self, count: usize);

    /// Returns the number of items available for an iterator.
    fn available(&mut self) -> usize;

    /// Returns the index of the iterator.
    fn index(&self) -> usize;

    /// Returns the length of the buffer.
    fn buf_len(&self) -> usize;
}

pub(crate) trait PrivateMRBIterator<T> {
    /// Sets the global index of this iterator.
    fn set_index(&self, index: usize);

    /// Returns the global index of successor.
    fn succ_index(&self) -> usize;

    /// Checks whether the current index can be returned
    fn check(&mut self, count: usize) -> bool;

    /// Returns Some(current element), if `check()` returns `true`, else None
    fn next(&mut self) -> Option<T>;

    /// Returns Some(&UnsafeSyncCell<current element>), if `check()` returns `true`, else None
    fn next_ref(&mut self) -> Option<&T>;

    /// Returns Some(&UnsafeSyncCell<current element>), if `check()` returns `true`, else None
    fn next_ref_mut(&mut self) -> Option<&mut T>;

    /// As next_ref_mut, but can be used for initialisation of inner MaybeUninit.
    fn next_ref_mut_init(&mut self) -> Option<*mut T>;

    fn next_chunk(&mut self, count: usize) -> Option<(&[T], &[T])>;

    fn next_chunk_mut(&mut self, count: usize) -> Option<(&mut [T], &mut [T])>;
}

pub(crate) mod macros {
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

    macro_rules! public_impl { () => (
        #[inline]
        unsafe fn advance(&mut self, count: usize) {
            self.index = match self.index >= self.buf_len - count {
                true => self.index + count - self.buf_len,
                false => self.index + count
            };

            self.set_index(self.index);
        }

        #[inline]
        fn index(&self) -> usize {
            self.index
        }

        #[inline]
        fn buf_len(&self) -> usize {
            self.buf_len
        }
    )}

    macro_rules! private_impl { () => (
        #[inline]
        fn check(&mut self, count: usize) -> bool {
            if self.cached_avail >= count || self.available() >= count {
                self.cached_avail -= count;
                true
            } else {
                false
            }
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
        fn next_ref(&mut self) -> Option<&T> {
            unsafe { self.check(1).then(|| self.buffer.inner()[self.index].inner_ref()) }
        }

        #[inline]
        fn next_ref_mut(&mut self) -> Option<&mut T> {
            unsafe { self.check(1).then(|| self.buffer.inner()[self.index].inner_ref_mut()) }
        }

        #[inline]
        fn next_ref_mut_init(&mut self) -> Option<*mut T> {
            self.check(1).then(|| self.buffer.inner()[self.index].as_mut_ptr())
        }

        #[inline]
        fn next_chunk(&mut self, count: usize) -> Option<(&[T], &[T])> {
            self.check(count).then(|| {
                let inner = unsafe {
                    transmute::<&[UnsafeSyncCell<T>], &[T]>(
                        slice::from_raw_parts(self.buffer.inner().as_ptr(), self.buf_len)
                    )
                };

                if self.index + count >= self.buf_len {
                    let (left, right) = inner.split_at(self.index);

                    (
                        right,
                        &left[.. self.index + count - self.buf_len]
                    )
                } else {
                    (&inner[self.index .. self.index + count], &[] as &[T])
                }
            })
        }

        #[inline]
        fn next_chunk_mut(&mut self, count: usize) -> Option<(&mut [T], &mut [T])> {
            self.check(count).then(|| {

                let inner = unsafe {
                    transmute::<&mut [UnsafeSyncCell<T>], &mut [T]>(
                        slice::from_raw_parts_mut(self.buffer.inner_mut().as_mut_ptr(), self.buf_len)
                    )
                };

                if self.index + count >= self.buf_len {
                    let (left, right) = inner.split_at_mut(self.index);

                    (
                        right,
                        &mut left[.. self.index + count - self.buf_len]
                    )
                } else {
                    (&mut inner[self.index .. self.index + count], &mut [] as &mut [T])
                }
            })
        }
    )}

    pub(crate) use { public_impl, private_impl, prod_alive, work_alive, cons_alive };
}