#[cfg(doc)]
use crate::iterators::ProdIter;

use crate::{ConcurrentMutRingBuf, LocalMutRingBuf, StackStorage, UnsafeSyncCell};

macro_rules! impl_rb {
    ($t: tt) => {
        impl<T, const N: usize> From<[T; N]> for $t<T, N> {
            #[doc = concat!("Converts an array into a [`", stringify!($t), "`].")]
            fn from(value: [T; N]) -> Self {
                Self::_from(StackStorage::from(value))
            }
        }

        impl<T, const N: usize> $t<T, N> {
            #[doc = concat!("Creates a new [`", stringify!($t), "`] with given capacity and zeroed (uninitialised) elements.")]
            /// # Safety
            /// The buffer must be then initialised using proper [`ProdIter`] methods (`*_init` ones).
            pub unsafe fn new_zeroed() -> Self {
                let v: [UnsafeSyncCell<T>; N] = core::array::from_fn(|_| UnsafeSyncCell::new_zeroed());

                Self::_from(StackStorage::from(v))
            }
        }

        impl<T: Default + Copy, const N: usize> Default for $t<T, N> {
            #[doc = concat!("Creates a new [`", stringify!($t), "`] with given capacity and elements initialised to `default`.")]
            fn default() -> Self {
                Self::from([T::default(); N])
            }
        }
    };
}

/// A stack-allocated ring buffer usable in concurrent environment.
pub type ConcurrentStackRB<T, const N: usize> = ConcurrentMutRingBuf<StackStorage<T, N>>;

impl_rb!(ConcurrentStackRB);

/// A stack-allocated ring buffer usable in local environment.
pub type LocalStackRB<T, const N: usize> = LocalMutRingBuf<StackStorage<T, N>>;

impl_rb!(LocalStackRB);
