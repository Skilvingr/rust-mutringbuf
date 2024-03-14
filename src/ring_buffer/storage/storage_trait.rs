use core::ops::Index;

use crate::UnsafeSyncCell;

#[allow(clippy::len_without_is_empty)]
pub trait Storage<T>: Index<usize, Output = UnsafeSyncCell<T>>
{
    fn as_ptr(&self) -> *const Self::Output;
    fn as_mut_ptr(&mut self) -> *mut Self::Output;
    fn len(&self) -> usize;
}
