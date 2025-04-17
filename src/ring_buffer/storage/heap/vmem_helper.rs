#![cfg_attr(doc, doc(cfg(feature = "vmem")))]
#![cfg(any(doc, feature = "vmem"))]

//! Utilities for `vmem` optimisation.

use core::ptr;
use crate::UnsafeSyncCell;

/// Returns the page size in use by the system.
#[cfg(unix)]
pub fn page_size() -> usize {
    unsafe {
        libc::sysconf(libc::_SC_PAGESIZE) as usize
    }
}

/// Returns a multiple of the page size in use by the system.
#[cfg(unix)]
pub fn get_page_size_mul(min_size: usize) -> usize {
    let page_size = page_size();
    min_size.div_ceil(page_size) * page_size
}

pub(crate) fn new<T>(value: &[UnsafeSyncCell<T>]) -> *mut UnsafeSyncCell<T> {
    let page_size = page_size();
    assert_eq!(value.len() % page_size, 0, "must be a multiple of page size, which is: {}.", page_size);
    
    unsafe {
        let size = size_of_val(value);
        
        let buffer = libc::mmap(
            ptr::null_mut(),
            2 * size as libc::size_t,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1, 0
        );
        
        libc::mmap(
            buffer.byte_add(size),
            size as libc::size_t,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_FIXED,
            -1, 0
        );

        let r = buffer as *mut UnsafeSyncCell<T>;
        libc::memcpy(value.as_ptr() as _, r as _, size_of_val(value));
        
        r
    }
}