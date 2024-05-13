use crate::{ConcurrentStackRB, LocalStackRB, StackStorage, UnsafeSyncCell};
#[allow(unused_imports)]
use crate::ProdIter;

pub mod concurrent_rb;
pub mod local_rb;
pub mod ring_buffer_trait;
pub mod alloc_ext;

impl<T, const N: usize> ConcurrentStackRB<T, N> {
    /// Creates a new concurrent stack-allocated buffer with given capacity and zeroed (uninitialised) elements.
    /// # Safety
    /// The buffer must be then initialised using proper [`ProdIter`] methods (`*_init` ones).
    pub unsafe fn new_zeroed() -> Self {
        let v: [UnsafeSyncCell<T>; N] = core::array::from_fn(|_| UnsafeSyncCell::new_zeroed());

        Self::_from(StackStorage::from(v))
    }
}

impl<T, const N: usize> LocalStackRB<T, N> {
    /// Creates a new local stack-allocated buffer with given capacity and zeroed (uninitialised) elements.
    /// # Safety
    /// The buffer must be then initialised using proper [`ProdIter`] methods (`*_init` ones).
    pub unsafe fn new_zeroed() -> Self {
        let v: [UnsafeSyncCell<T>; N] = core::array::from_fn(|_| UnsafeSyncCell::new_zeroed());

        Self::_from(StackStorage::from(v))
    }
}