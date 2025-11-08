#![cfg_attr(doc, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[doc(inline)]
pub use iterators::MRBIterator;

pub use ring_buffer::variants::ring_buffer_trait::{ConcurrentRB, MutRB};
pub use ring_buffer::wrappers::unsafe_sync_cell::UnsafeSyncCell;

#[cfg_attr(doc, doc(cfg(all(feature = "alloc", feature = "async"))))]
#[cfg(all(feature = "async", feature = "alloc"))]
pub use crate::ring_buffer::storage::heap::rb::AsyncHeapRB;

#[cfg_attr(doc, doc(cfg(all(not(feature = "vmem"), feature = "async"))))]
#[cfg(any(all(not(feature = "vmem"), feature = "async"), doc))]
pub use crate::ring_buffer::storage::stack::rb::AsyncStackRB;

#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
pub use crate::ring_buffer::storage::heap::{
    HeapSplit, HeapStorage,
    rb::{ConcurrentHeapRB, LocalHeapRB},
};

#[cfg_attr(doc, doc(cfg(not(feature = "vmem"))))]
#[cfg(any(not(feature = "vmem"), doc))]
pub use crate::ring_buffer::storage::stack::{
    StackSplit, StackStorage,
    rb::{ConcurrentStackRB, LocalStackRB},
};

#[cfg_attr(doc, doc(cfg(feature = "vmem")))]
#[cfg(any(doc, feature = "vmem"))]
pub use crate::ring_buffer::storage::heap::vmem_helper;

pub use crate::ring_buffer::storage::Storage;

#[cfg_attr(doc, doc(cfg(feature = "async")))]
#[cfg(feature = "async")]
pub use crate::ring_buffer::variants::async_rb::AsyncMutRingBuf;
pub use crate::ring_buffer::variants::concurrent_rb::ConcurrentMutRingBuf;
pub use crate::ring_buffer::variants::local_rb::LocalMutRingBuf;

pub mod iterators;
mod ring_buffer;
