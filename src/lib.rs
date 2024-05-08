#![doc = include_str!("../README.md")]

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use iterators::impls::prod_iter::ProdIter;
pub use iterators::impls::work_iter::WorkIter;
pub use iterators::impls::detached_work_iter::DetachedWorkIter;
pub use iterators::impls::cons_iter::ConsIter;
pub use iterators::iterator_trait::MRBIterator;
pub use ring_buffer::variants::ring_buffer_trait::MutRB;
pub use ring_buffer::wrappers::unsafe_sync_cell::UnsafeSyncCell;

#[cfg(feature = "alloc")]
pub use crate::ring_buffer::storage::heap::HeapStorage;
pub use crate::ring_buffer::storage::stack::StackStorage;

pub use crate::ring_buffer::variants::local_rb::LocalMutRingBuf;
pub use crate::ring_buffer::variants::concurrent_rb::ConcurrentMutRingBuf;


mod iterators;
mod ring_buffer;


// Concurrent buffer types

/// A stack-allocated ring buffer usable in concurrent environment.
pub type ConcurrentStackRB<T, const N: usize> = ConcurrentMutRingBuf<StackStorage<T, N>>;

/// A heap-allocated ring buffer usable in concurrent environment.
#[cfg(feature = "alloc")]
pub type ConcurrentHeapRB<T> = ConcurrentMutRingBuf<HeapStorage<T>>;


// Local buffer types

/// A stack-allocated ring buffer usable in local environment.
pub type LocalStackRB<T, const N: usize> = LocalMutRingBuf<StackStorage<T, N>>;

/// A heap-allocated ring buffer usable in local environment.
#[cfg(feature = "alloc")]
pub type LocalHeapRB<T> = LocalMutRingBuf<HeapStorage<T>>;

