use crate::ring_buffer::variants::ring_buffer_trait::{IterManager, PrivateIterManager};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr::NonNull;

pub struct BufRef<'buf, B> {
    inner: NonNull<B>,
    needs_drop: bool,
    _phantom: PhantomData<&'buf ()>,
}

#[allow(private_bounds)]
impl<'buf, B: IterManager + PrivateIterManager> BufRef<'buf, B> {
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
    #[inline(never)]
    fn try_drop(&mut self) {
        if self.needs_drop {
            unsafe {
                let _ = Box::from_raw(self.inner.as_ptr());
            }
        }
    }

    #[cfg(not(feature = "vmem"))]
    pub(crate) fn from_ref(buf: &'buf mut B) -> Self {
        Self {
            inner: NonNull::from(buf),
            needs_drop: false,
            _phantom: Default::default(),
        }
    }
}

impl<B> Clone for BufRef<'_, B> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner,
            needs_drop: self.needs_drop,
            _phantom: Default::default(),
        }
    }
}

impl<B> Deref for BufRef<'_, B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}

#[allow(private_bounds)]
impl<B: IterManager + PrivateIterManager> BufRef<'_, B> {
    pub(crate) fn drop_iter(&mut self) {
        if unsafe { self.inner.as_ref().drop_iter() } != 1 {
            return;
        }

        self.acquire_fence();

        self.try_drop();
    }
}
