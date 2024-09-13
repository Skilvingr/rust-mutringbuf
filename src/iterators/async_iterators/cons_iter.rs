use crate::ConsIter;
use crate::iterators::async_iterators::async_macros::{futures_import, gen_common_futs, gen_fut, waker_registerer};
use crate::iterators::iterator_trait::MRBIterator;
use crate::iterators::util_macros::delegate;
use crate::iterators::util_macros::muncher;
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, MutRB};

futures_import!();

#[doc = r##"
Async version of [`ConsIter`].
"##]

pub struct AsyncConsIter<'buf, B: MutRB, const W: bool> {
    inner: ConsIter<'buf, B, W>,
    waker: Option<Waker>
}
unsafe impl<'buf, B: ConcurrentRB + MutRB<Item = T>, T, const W: bool> Send for AsyncConsIter<'buf, B, W> {}

gen_common_futs!(&'a mut AsyncConsIter<'buf, B, W>, (const W: bool));

gen_fut!{
    PeekRefFuture<'a, B: MutRB<Item = T>, T: 'a, (const W: bool)>,
    &'a mut AsyncConsIter<'buf, B, W>,
    (),
    Option<&'a T>,
    self {
        let result = self.iter.inner.peek_ref();

        if let Some(res) = result {
            break Poll::Ready(Some(res));
        }
    }
}

gen_fut!{
    PeekSliceFuture<'a, B: MutRB<Item = T>, T: 'a, (const W: bool)>,
    &'a mut AsyncConsIter<'buf, B, W>,
    usize,
    Option<(&'a [T], &'a [T])>,
    self {
        let count = self._item;
        let result = self.iter.inner.peek_slice(count);

        if let Some(res) = result {
            break Poll::Ready(Some(res));
        }
    }
}

gen_fut!{
    PeekAvailableFuture<'a, B: MutRB<Item = T>, T: 'a, (const W: bool)>,
    &'a mut AsyncConsIter<'buf, B, W>,
    (),
    Option<(&'a [T], &'a [T])>,
    self {
        let result = self.iter.inner.peek_available();

        if let Some(res) = result {
            break Poll::Ready(Some(res));
        }
    }
}

gen_fut!{
    PopFuture<'a, B: MutRB<Item = T>, T: 'a, (const W: bool)>,
    &'a mut AsyncConsIter<'buf, B, W>,
    (),
    Option<T>,
    self {
        let result = self.iter.inner.pop();

        if let Some(res) = result {
            break Poll::Ready(Some(res));
        }
    }
}

gen_fut!{
    PopMoveFuture<'a, B: MutRB<Item = T>, T: 'a, (const W: bool)>,
    &'a mut AsyncConsIter<'buf, B, W>,
    (),
    Option<T>,
    self {
        let result = unsafe { self.iter.inner.pop_move() };

        if let Some(res) = result {
            break Poll::Ready(Some(res));
        }
    }
}

gen_fut!{
    CopyItemFuture<'a, 'b, B: MutRB<Item = T>, T: Copy, (const W: bool)>,
    &'a mut AsyncConsIter<'buf, B, W>,
    Option<&'b mut T>,
    (),
    self {
        let dst = self._item.take().unwrap();
        let result = self.iter.inner.copy_item(dst);

        if result.is_some() {
            break Poll::Ready(());
        }
    }
}

gen_fut!{
    CloneItemFuture<'a, 'b, B: MutRB<Item = T>, T: Clone, (const W: bool)>,
    &'a mut AsyncConsIter<'buf, B, W>,
    Option<&'b mut T>,
    (),
    self {
        let dst = self._item.take().unwrap();
        let result = self.iter.inner.clone_item(dst);

        if result.is_some() {
            break Poll::Ready(());
        }
    }
}

gen_fut!{
    CopySliceFuture<'a, 'b, B: MutRB<Item = T>, T: Copy, (const W: bool)>,
    &'a mut AsyncConsIter<'buf, B, W>,
    Option<&'b mut [T]>,
    (),
    self {
        let dst = self._item.take().unwrap();
        let result = self.iter.inner.copy_slice(dst);

        if result.is_some() {
            break Poll::Ready(());
        }
    }
}

gen_fut!{
    CloneSliceFuture<'a, 'b, B: MutRB<Item = T>, T: Clone, (const W: bool)>,
    &'a mut AsyncConsIter<'buf, B, W>,
    Option<&'b mut [T]>,
    (),
    self {
        let dst = self._item.take().unwrap();
        let result = self.iter.inner.clone_slice(dst);

        if result.is_some() {
            break Poll::Ready(());
        }
    }
}


impl<'buf, B: MutRB<Item = T>, T, const W: bool> AsyncConsIter<'buf, B, W> {
    pub fn from_sync(iter: ConsIter<'buf, B, W>) -> Self {
        Self {
            inner: iter,
            waker: None,
        }
    }

    pub fn into_sync(self) -> ConsIter<'buf, B, W> {
        self.inner
    }

    waker_registerer!();
    delegate!(ConsIter, pub fn is_prod_alive(&self) -> bool);
    delegate!(ConsIter, pub fn is_work_alive(&self) -> bool);
    delegate!(ConsIter, pub fn prod_index(&self) -> usize);
    delegate!(ConsIter, pub fn work_index(&self) -> usize);
    delegate!(ConsIter, pub fn index(&self) -> usize);
    delegate!(ConsIter, pub unsafe fn advance(&(mut) self, count: usize));
    delegate!(ConsIter, pub fn available(&(mut) self) -> usize);
    delegate!(ConsIter, pub fn reset_index(&(mut) self));


    pub fn peek_ref(&mut self) -> PeekRefFuture<'buf, '_, B, T, W> {
        PeekRefFuture {
            iter: self,
            _item: (),
        }
    }

    pub fn peek_slice(&mut self, count: usize) -> PeekSliceFuture<'buf, '_, B, T, W> {
        PeekSliceFuture {
            iter: self,
            _item: count,
        }
    }

    pub fn peek_available(&mut self) -> PeekAvailableFuture<'buf, '_, B, T, W> {
        PeekAvailableFuture {
            iter: self,
            _item: (),
        }
    }

    /// Tries to pop an element, moving it.
    /// # Safety
    /// Same as [`ConsIter::pop`]
    pub fn pop(&mut self) -> PopFuture<'buf, '_, B, T, W> {
        PopFuture {
            iter: self,
            _item: (),
        }
    }

    /// Tries to pop an element, moving it.
    /// # Safety
    /// Same as [`ConsIter::pop`]
    pub unsafe fn pop_move(&mut self) -> PopMoveFuture<'buf, '_, B, T, W> {
        PopMoveFuture {
            iter: self,
            _item: (),
        }
    }

    pub fn copy_item<'b>(&mut self, dst: &'b mut T) -> CopyItemFuture<'buf, '_, 'b, B, T, W>
        where T: Copy
    {
        CopyItemFuture {
            iter: self,
            _item: Some(dst),
        }
    }

    #[inline]
    pub fn clone_item<'b>(&mut self, dst: &'b mut T) -> CloneItemFuture<'buf, '_, 'b, B, T, W>
        where T: Clone
    {
        CloneItemFuture {
            iter: self,
            _item: Some(dst),
        }
    }

    pub fn copy_slice<'b>(&mut self, dst: &'b mut [T]) -> CopySliceFuture<'buf, '_, 'b, B, T, W>
        where T: Copy
    {
        CopySliceFuture {
            iter: self,
            _item: Some(dst),
        }
    }

    pub fn clone_slice<'b>(&mut self, dst: &'b mut [T]) -> CloneSliceFuture<'buf, '_, 'b, B, T, W>
        where T: Clone
    {
        CloneSliceFuture {
            iter: self,
            _item: Some(dst),
        }
    }
}
