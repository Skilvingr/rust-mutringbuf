#![cfg_attr(doc, doc(cfg(feature = "vmem")))]
#![cfg(any(doc, feature = "vmem"))]

//! Utilities for `vmem` optimisation.

use crate::UnsafeSyncCell;
use core::ptr;
use libc::c_int;

/// Returns the page size in use by the system.
#[cfg(unix)]
pub fn page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
}

/// Returns a multiple of the page size in use by the system.
#[cfg(unix)]
pub fn get_page_size_mul(min_size: usize) -> usize {
    let page_size = page_size();
    min_size.div_ceil(page_size) * page_size
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
unsafe fn open_fd() -> c_int {
    use alloc::ffi::CString;
    use alloc::format;
    use libc::rand;

    let group_name = option_env!("IOS_APP_GROUP_NAME");

    libc::srand(libc::time(ptr::null_mut()) as libc::c_uint);
    let mut name;

    let fd = loop {
        name = CString::new(if let Some(gn) = group_name {
            format!("{}{}{}", gn, "/mrb", rand() % 99)
        } else {
            format!("{}{}", "/mrb", rand() % 99)
        })
        .unwrap();
        let fd = libc::shm_open(
            name.as_ptr(),
            libc::O_CREAT | libc::O_RDWR | libc::O_EXCL,
            0700,
        );

        if fd != -1 || *libc::__error() != libc::EEXIST {
            break fd;
        }
    };

    assert_eq!(libc::shm_unlink(name.as_ptr()), 0, "shm_unlink failed");

    fd
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
unsafe fn open_fd() -> c_int {
    unsafe { libc::memfd_create(c"/mrb".as_ptr(), 0) }
}

pub(crate) fn new<T>(value: &[UnsafeSyncCell<T>]) -> *mut UnsafeSyncCell<T> {
    let page_size = page_size();
    let size = size_of_val(value);

    assert_eq!(
        size % page_size,
        0,
        "the size of the buffer (len * size_of::<T>()) must be a multiple of page size, which is: {}.",
        page_size
    );

    unsafe {
        // The real place where the buffer is allocated
        let fd = open_fd();

        assert_ne!(fd, -1, "shared fd creation failed");

        if libc::ftruncate(fd, size as libc::off_t) == -1 {
            assert_eq!(libc::close(fd), 0, "close failed");
            panic!("ftruncate failed");
        }

        // Reserve a block double the size of the buffer
        let buffer = libc::mmap(
            ptr::null_mut(),
            2 * size as libc::size_t,
            libc::PROT_NONE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        );
        assert_ne!(buffer as isize, -1, "mmap 1 failed");

        // Map the first part of the previously reserved memory to the fd.
        // Regarding the reserved memory, the overlapping part is automatically unmapped.
        let addr = libc::mmap(
            buffer,
            size as libc::size_t,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED | libc::MAP_FIXED,
            fd,
            0,
        );
        assert_ne!(addr as isize, -1, "mmap 2 failed");

        // Map the second part of the previously reserved memory to the fd.
        // Regarding the reserved memory, the overlapping part is automatically unmapped.
        let addr = libc::mmap(
            buffer.byte_add(size),
            size as libc::size_t,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED | libc::MAP_FIXED,
            fd,
            0,
        );
        assert_ne!(addr as isize, -1, "mmap 3 failed");

        assert_eq!(libc::close(fd), 0, "close failed");

        let r = buffer as *mut UnsafeSyncCell<T>;
        ptr::copy_nonoverlapping(value.as_ptr(), r, value.len());

        r
    }
}
