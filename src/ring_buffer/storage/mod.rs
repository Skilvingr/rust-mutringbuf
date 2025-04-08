use core::ops::Index;

use crate::UnsafeSyncCell;

pub mod heap;
pub mod stack;

#[allow(clippy::len_without_is_empty)]
pub trait Storage: Index<usize, Output = UnsafeSyncCell<Self::Item>>
{
    type Item;

    /// Returns the underlying array as a const ptr.
    fn as_ptr(&self) -> *const Self::Output;
    /// Returns the underlying array as a mutable ptr.
    fn as_mut_ptr(&mut self) -> *mut Self::Output;
    /// Returns the length of the underlying array.
    fn len(&self) -> usize;
}