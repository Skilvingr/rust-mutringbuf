#![doc = include_str!("../README.md")]

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use iterators::impls::prod_iter::ProdIter;
pub use iterators::impls::work_iter::WorkIter;
pub use iterators::impls::detached_work_iter::DetachedWorkIter;
pub use iterators::impls::cons_iter::ConsIter;
pub use iterators::iterator_trait::Iterator;
pub use ring_buffer::wrappers::unsafe_sync_cell::UnsafeSyncCell;

#[cfg(feature = "alloc")]
use crate::ring_buffer::storage::heap::HeapStorage;
use crate::ring_buffer::storage::stack::StackStorage;

use crate::ring_buffer::variants::local::LocalMutRingBuf;
use crate::ring_buffer::variants::concurrent::ConcurrentMutRingBuf;


pub mod iterators;
pub mod ring_buffer;


// Concurrent buffer types

/// A stack-allocated ring buffer usable in concurrent environment.
pub type ConcurrentStackRB<T, const N: usize> = ConcurrentMutRingBuf<StackStorage<T, N>, T>;

/// A heap-allocated ring buffer usable in concurrent environment.
#[cfg(feature = "alloc")]
pub type ConcurrentHeapRB<T> = ConcurrentMutRingBuf<HeapStorage<T>, T>;


// Local buffer types

/// A stack-allocated ring buffer usable in local environment.
pub type LocalStackRB<T, const N: usize> = LocalMutRingBuf<StackStorage<T, N>, T>;

/// A heap-allocated ring buffer usable in local environment.
#[cfg(feature = "alloc")]
pub type LocalHeapRB<T> = LocalMutRingBuf<HeapStorage<T>, T>;

