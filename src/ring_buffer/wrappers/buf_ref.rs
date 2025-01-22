use crate::ring_buffer::variants::ring_buffer_trait::IterManager;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr::NonNull;
use core::sync::atomic::fence;
use core::sync::atomic::Ordering::SeqCst;

pub struct BufRef<'buf, B> {
    inner: NonNull<B>,
    needs_drop: bool,
    _phantom: PhantomData<&'buf ()>
}

impl<'buf, B> BufRef<'buf, B> {
    #[cfg(feature = "alloc")]
    pub(crate) fn new(buf: B) -> Self {
        let x = Box::new(buf);

        Self {
            inner: NonNull::new(Box::into_raw(x)).unwrap(),
            needs_drop: true,
            _phantom: Default::default(),
        }
    }
    
    #[cfg(feature = "alloc")]
    pub(crate) fn drop(&mut self) {
        if self.needs_drop {
            unsafe { let _ = Box::from_raw(self.inner.as_ptr()); }
        }
    }

    pub(crate) fn from_ref(buf: &'buf mut B) -> Self {
        Self {
            inner: NonNull::from(buf),
            needs_drop: false,
            _phantom: Default::default(),
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub(crate) fn drop(&self) {}
}

impl<'buf, B> Clone for BufRef<'buf, B> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner,
            needs_drop: self.needs_drop,
            _phantom: Default::default(),
        }
    }
}

impl<'buf, B> Deref for BufRef<'buf, B> {
    type Target = B;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}


impl<'buf, B: IterManager> BufRef<'buf, B> {
    pub(crate) fn set_prod_alive(&mut self, alive: bool) {
        unsafe {
            fence(SeqCst);
            self.inner.as_ref().set_prod_alive(alive);

            let cond = !self.inner.as_ref().work_alive() && !self.inner.as_ref().cons_alive();
            fence(SeqCst);

            if cond {
                self.drop();
            }
        }
    }

    pub(crate) fn set_work_alive(&mut self, alive: bool) {
        unsafe {
            fence(SeqCst);
            self.inner.as_ref().set_work_alive(alive);

            let cond = !self.inner.as_ref().prod_alive() && !self.inner.as_ref().cons_alive();
            fence(SeqCst);

            if cond {
                self.drop();
            }
        }
    }

    pub(crate) fn set_cons_alive(&mut self, alive: bool) {
        unsafe {
            fence(SeqCst);
            self.inner.as_ref().set_cons_alive(alive);

            let cond = !self.inner.as_ref().prod_alive() && !self.inner.as_ref().work_alive();
            fence(SeqCst);

            if cond {
                self.drop();
            }
        }
    }
}