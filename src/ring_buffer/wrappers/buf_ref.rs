use crate::ring_buffer::variants::ring_buffer_trait::{IterManager, PrivateIterManager};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr::NonNull;

pub struct BufRef<'buf, B, const NEEDS_DROP: bool> {
    inner: NonNull<B>,
    _phantom: PhantomData<&'buf ()>,
}

impl<'buf, B: IterManager + PrivateIterManager, const NEEDS_DROP: bool>
    BufRef<'buf, B, NEEDS_DROP>
{
    #[cfg(any(not(feature = "vmem"), doc))]
    pub(crate) fn from_ref(buf: &'buf mut B) -> BufRef<'buf, B, false> {
        BufRef::<'buf, B, false> {
            inner: NonNull::from(buf),
            _phantom: Default::default(),
        }
    }

    pub(crate) fn new(buf: B) -> BufRef<'buf, B, true> {
        let x = Box::new(buf);

        BufRef::<'buf, B, true> {
            inner: NonNull::new(Box::into_raw(x)).unwrap(),
            _phantom: Default::default(),
        }
    }
}

pub(crate) trait RefDropManager {
    fn drop_iter(&mut self);
    fn try_drop(&mut self);
}
impl<B: IterManager + PrivateIterManager, const NEEDS_DROP: bool> RefDropManager
    for BufRef<'_, B, NEEDS_DROP>
{
    fn drop_iter(&mut self) {
        if unsafe { self.inner.as_ref().decrement_iter_counter() } != 1 {
            return;
        }

        self.acquire_fence();

        self.try_drop();
    }

    #[inline(never)]
    fn try_drop(&mut self) {
        if NEEDS_DROP {
            unsafe {
                let _ = Box::from_raw(self.inner.as_ptr());
            }
        }
    }
}

impl<B, const NEEDS_DROP: bool> Clone for BufRef<'_, B, NEEDS_DROP> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner,
            _phantom: Default::default(),
        }
    }
}

impl<B, const NEEDS_DROP: bool> Deref for BufRef<'_, B, NEEDS_DROP> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}
