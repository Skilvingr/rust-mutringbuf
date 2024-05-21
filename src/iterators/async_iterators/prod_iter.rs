use crate::iterators::async_iterators::async_macros::{futures_import, gen_fut, waker_registerer};
use crate::iterators::iterator_trait::MRBIterator;
use crate::iterators::util_macros::delegate;
use crate::iterators::util_macros::muncher;
use crate::ProdIter;
use crate::ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, MutRB};

futures_import!();

#[doc = r##"
Async version of [`ProdIter`].
"##]

pub struct AsyncProdIter<B: MutRB> {
    inner: ProdIter<B>,
    waker: Option<Waker>
}
unsafe impl<B: ConcurrentRB + MutRB<Item = T>, T> Send for AsyncProdIter<B> {}


gen_fut!{
    PushFuture<'a, B: MutRB<Item = T>, T>,
    &'a mut AsyncProdIter<B>,
    Option<T>,
    (),
    self {
        let item = self._item.take().unwrap();
        let push_result = self.iter.inner.push(item);

        if push_result.is_ok() {
            break Poll::Ready(());
        } else {
            self._item = Some(push_result.unwrap_err());
        }
    }
}

gen_fut!{
    PushInitFuture<'a, B: MutRB<Item = T>, T>,
    &'a mut AsyncProdIter<B>,
    Option<T>,
    (),
    self {
        let item = self._item.take().unwrap();
        let push_result = self.iter.inner.push_init(item);

        if push_result.is_ok() {
            break Poll::Ready(());
        } else {
            self._item = Some(push_result.unwrap_err());
        }
    }
}

gen_fut!{
    PushSliceFuture<'a, 'b, B: MutRB<Item = T>, T: Copy>,
    &'a mut AsyncProdIter<B>,
    &'b [T],
    (),
    self {
        let item = self._item;
        let push_result = self.iter.inner.push_slice(item);

        if push_result.is_some() {
            break Poll::Ready(());
        }
    }
}

gen_fut!{
    PushSliceInitFuture<'a, 'b, B: MutRB<Item = T>, T: Copy>,
    &'a mut AsyncProdIter<B>,
    &'b [T],
    (),
    self {
        let item = self._item;
        let push_result = self.iter.inner.push_slice_init(item);

        if push_result.is_some() {
            break Poll::Ready(());
        }
    }
}

gen_fut!{
    PushSliceCloneFuture<'a, 'b, B: MutRB<Item = T>, T: Clone>,
    &'a mut AsyncProdIter<B>,
    &'b [T],
    (),
    self {
        let item = self._item;
        let push_result = self.iter.inner.push_slice_clone(item);

        if push_result.is_some() {
            break Poll::Ready(());
        }
    }
}

gen_fut!{
    PushSliceCloneInitFuture<'a, 'b, B: MutRB<Item = T>, T: Clone>,
    &'a mut AsyncProdIter<B>,
    &'b [T],
    (),
    self {
        let item = self._item;
        let push_result = self.iter.inner.push_slice_clone_init(item);

        if push_result.is_some() {
            break Poll::Ready(());
        }
    }
}

gen_fut!{
    GetNextItemMutFuture<'a, B: MutRB<Item = T>, T: 'a>,
    &'a mut AsyncProdIter<B>,
    (),
    &'a mut T,
    self {
        let push_result = unsafe { self.iter.inner.get_next_item_mut() };

        if let Some(res) = push_result {
            break Poll::Ready(res);
        }
    }
}


gen_fut!{
    GetNextItemMutInitFuture<'a, B: MutRB<Item = T>, T>,
    &'a mut AsyncProdIter<B>,
    (),
    *mut T,
    self {
        let push_result = self.iter.inner.get_next_item_mut_init();

        if let Some(res) = push_result {
            break Poll::Ready(res);
        }
    }
}


impl<B: MutRB<Item = T>, T> AsyncProdIter<B> {
    pub fn from_sync(iter: ProdIter<B>) -> Self {
        Self {
            inner: iter,
            waker: None,
        }
    }

    pub fn into_sync(self) -> ProdIter<B> {
        self.inner
    }

    waker_registerer!();
    delegate!(ProdIter, pub fn is_work_alive(&self) -> bool);
    delegate!(ProdIter, pub fn is_cons_alive(&self) -> bool);
    delegate!(ProdIter, pub fn work_index(&self) -> usize);
    delegate!(ProdIter, pub fn cons_index(&self) -> usize);
    delegate!(ProdIter, pub fn index(&self) -> usize);
    delegate!(ProdIter, pub fn available(&(mut) self) -> usize);

    /// Async version of [`ProdIter::push`].
    #[inline]
    pub fn push(&mut self, value: T) -> PushFuture<'_, B, T> {
        PushFuture {
            iter: self,
            _item: Some(value),
        }
    }

    /// Async version of [`ProdIter::push_slice`].
    #[inline]
    pub fn push_slice<'b>(&mut self, slice: &'b [T]) -> PushSliceFuture<'_, 'b, B, T>
        where T: Copy
    {
        PushSliceFuture {
            iter: self,
            _item: slice,
        }
    }

    /// Async version of [`ProdIter::push_slice_clone`].
    #[inline]
    pub fn push_slice_clone<'b>(&mut self, slice: &'b [T]) -> PushSliceCloneFuture<'_, 'b, B, T>
        where T: Clone
    {
       PushSliceCloneFuture {
           iter: self,
           _item: slice,
       }
    }

    /// Async version of [`ProdIter::get_next_item_mut`].
    /// # Safety
    /// Same as [`ProdIter::get_next_item_mut`].
    pub unsafe fn get_next_item_mut(&mut self) -> GetNextItemMutFuture<'_, B, T> {
        GetNextItemMutFuture {
            iter: self,
            _item: ()
        }
    }

    /// Async version of [`ProdIter::get_next_item_mut_init`].
    pub fn get_next_item_mut_init(&mut self) -> GetNextItemMutInitFuture<'_, B, T> {
        GetNextItemMutInitFuture {
            iter: self,
            _item: ()
        }
    }
}