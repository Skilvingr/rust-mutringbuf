use crate::HeapStorage;
#[allow(unused_imports)]
use crate::iterators::ProdIter;
#[cfg(any(feature = "async", doc))]
use crate::ring_buffer::variants::async_rb::AsyncMutRingBuf;
use crate::{ConcurrentMutRingBuf, LocalMutRingBuf, UnsafeSyncCell};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

fn get_range_max(capacity: usize) -> usize {
    #[cfg(feature = "vmem")]
    return super::vmem_helper::get_page_size_mul(capacity);
    #[cfg(not(feature = "vmem"))]
    return capacity;
}

macro_rules! impl_rb {
    ($t: tt) => {
        impl<T> From<Vec<T>> for $t<T> {
            #[doc = concat!("Converts a `Vec<T>` into a [`", stringify!($t), "`].")]
            /// Note that the length of the buffer will be equal to the length of the vector, and *not*
            /// to its capacity.
            /// # Behaviour with `vmem` feature
            /// When `vmem` feature is enabled, the capacity of the buffer must be a multiple of
            /// the system's page size, so must be the length of the passed `Vec`.
            /// Please, use [`crate::vmem_helper::get_page_size_mul`] to get a suitable length.
            fn from(value: Vec<T>) -> Self {
                Self::_from(HeapStorage::from(value))
            }
        }

        impl<T> $t<T> {
            #[doc = concat!("Creates a new [`", stringify!($t), "`] with given capacity and zeroed (uninitialised) elements.")]
            /// # Safety
            /// The buffer must be then initialised using proper [`ProdIter`] methods (`*_init` ones).
            /// # Behaviour with `vmem` feature
            /// When `vmem` feature is enabled, the capacity of the buffer must be a multiple of
            /// the system's page size.
            /// This method accepts a minimum size, which will then be used to compute the actual
            /// size (equal to or greater than it).
            pub unsafe fn new_zeroed(capacity: usize) -> Self {
                Self::_from(
                    HeapStorage::from(
                        (0..get_range_max(capacity))
                        .map(|_| UnsafeSyncCell::new_zeroed()).collect::<Box<[UnsafeSyncCell<T>]>>()
                    )
                )
            }

            #[doc = concat!("Creates a new [`", stringify!($t), "`] with given capacity and elements initialised to `default`.")]
            /// # Behaviour with `vmem` feature
            /// When `vmem` feature is enabled, the capacity of the buffer must be a multiple of
            /// the system's page size.
            /// This method accepts a minimum size, which will then be used to compute the actual
            /// size (equal to or greater than it).
            pub fn default(capacity: usize) -> Self
                where T: Default + Clone {
                Self::from(vec![T::default(); get_range_max(capacity)])
            }
        }
    };
}

// Concurrent

/// A stack-allocated asynchronous ring buffer usable in concurrent environment.
#[cfg(any(feature = "async", doc))]
pub type AsyncHeapRB<T> = AsyncMutRingBuf<HeapStorage<T>>;
#[cfg(any(feature = "async", doc))]
impl_rb!(AsyncHeapRB);

/// A heap-allocated ring buffer usable in a concurrent environment.
pub type ConcurrentHeapRB<T> = ConcurrentMutRingBuf<HeapStorage<T>>;

impl_rb!(ConcurrentHeapRB);

// Local

/// A heap-allocated ring buffer usable in a local environment.
pub type LocalHeapRB<T> = LocalMutRingBuf<HeapStorage<T>>;

impl_rb!(LocalHeapRB);
