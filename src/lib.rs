#![doc = include_str!("../README.md")]

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[doc(inline)]
pub use iterators::MRBIterator;

#[doc(inline)]
pub use iterators::sync_iterators::{
    cons_iter::ConsIter,
    prod_iter::ProdIter,
    work_iter::WorkIter,
    detached::Detached
};
#[cfg(feature = "alloc")]
pub use ring_buffer::variants::HeapSplit;
pub use ring_buffer::variants::ring_buffer_trait::{MutRB, ConcurrentRB};
pub use ring_buffer::variants::StackSplit;
pub use ring_buffer::wrappers::unsafe_sync_cell::UnsafeSyncCell;

#[doc(inline)]
#[cfg(any(feature = "async", doc))]
pub use iterators::async_iterators::{
    cons_iter::AsyncConsIter,
    prod_iter::AsyncProdIter,
    work_iter::AsyncWorkIter,
    detached::AsyncDetached
};
#[cfg(feature = "alloc")]
pub use crate::ring_buffer::{
    storage::heap::HeapStorage,
    variants::alloc_ext::{ConcurrentHeapRB, LocalHeapRB}
};
pub use crate::ring_buffer::{
    storage::stack::StackStorage,
    variants::{ConcurrentStackRB, LocalStackRB}
};
pub use crate::ring_buffer::storage::Storage;
pub use crate::ring_buffer::variants::concurrent_rb::ConcurrentMutRingBuf;
pub use crate::ring_buffer::variants::local_rb::LocalMutRingBuf;

pub mod iterators;
mod ring_buffer;
