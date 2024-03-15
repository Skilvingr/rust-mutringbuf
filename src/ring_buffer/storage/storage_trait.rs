use core::ops::Index;

use crate::UnsafeSyncCell;

#[allow(clippy::len_without_is_empty)]
pub trait Storage<T>: Index<usize, Output = UnsafeSyncCell<T>>
{
    /// Returns the underlying array as a const ptr.
    fn as_ptr(&self) -> *const Self::Output;
    /// Returns the underlying array as a mutable ptr.
    fn as_mut_ptr(&mut self) -> *mut Self::Output;
    /// Returns the length of the underlying array.
    fn len(&self) -> usize;
}
