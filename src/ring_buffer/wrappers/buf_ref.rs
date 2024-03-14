#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use crate::ring_buffer::variants::ring_buffer_trait::IterManager;

pub struct BufRef<B> {
    inner: NonNull<B>
}

impl<B> BufRef<B> {
    #[cfg(feature = "alloc")]
    pub(crate) fn new(buf: B) -> Self {
        let x = Box::new(buf);

        Self {
            inner: NonNull::new(Box::into_raw(x)).unwrap()
        }
    }

    #[cfg(feature = "alloc")]
    pub(crate) fn drop(&self) {
        unsafe { drop(Box::from_raw(self.inner.as_ptr())) };
    }

    #[cfg(not(feature = "alloc"))]
    pub(crate) fn new(mut buf: B) -> Self {
        Self {
            inner: NonNull::new(&mut buf as *mut B).unwrap()
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub(crate) fn drop(&self) {
        drop(self.inner.as_ptr());
    }
}

impl<B> Clone for BufRef<B> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner
        }
    }
}

impl<B> Deref for BufRef<B> {
    type Target = B;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}

impl<B> DerefMut for BufRef<B> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.inner.as_mut() }
    }
}

impl<B: IterManager> IterManager for BufRef<B> {
    #[inline]
    fn prod_index(&self) -> usize {
        unsafe { self.inner.as_ref().prod_index() }
    }

    #[inline]
    fn work_index(&self) -> usize {
        unsafe { self.inner.as_ref().work_index() }
    }

    #[inline]
    fn cons_index(&self) -> usize {
        unsafe { self.inner.as_ref().cons_index() }
    }

    #[inline]
    fn set_prod_index(&self, index: usize) {
        unsafe { self.inner.as_ref().set_prod_index(index) }
    }

    #[inline]
    fn set_work_index(&self, index: usize) {
        unsafe { self.inner.as_ref().set_work_index(index) }
    }

    #[inline]
    fn set_cons_index(&self, index: usize) {
        unsafe { self.inner.as_ref().set_cons_index(index) }
    }

    #[inline]
    fn prod_alive(&self) -> bool {
        unsafe { self.inner.as_ref().prod_alive() }
    }

    #[inline]
    fn work_alive(&self) -> bool {
        unsafe { self.inner.as_ref().work_alive() }
    }

    #[inline]
    fn cons_alive(&self) -> bool {
        unsafe { self.inner.as_ref().cons_alive() }
    }

    #[inline]
    fn set_prod_alive(&self, alive: bool) {
        unsafe {
            self.inner.as_ref().set_prod_alive(alive);

            if !self.inner.as_ref().work_alive() && !self.inner.as_ref().cons_alive() {
                self.drop();
            }
        }
    }

    #[inline]
    fn set_work_alive(&self, alive: bool) {
        unsafe {
            self.inner.as_ref().set_work_alive(alive);

            if !self.inner.as_ref().prod_alive() && !self.inner.as_ref().cons_alive() {
                self.drop();
            }
        }
    }

    #[inline]
    fn set_cons_alive(&self, alive: bool) {
        unsafe {
            self.inner.as_ref().set_cons_alive(alive);

            if !self.inner.as_ref().prod_alive() && !self.inner.as_ref().work_alive() {
                self.drop();
            }
        }
    }
}