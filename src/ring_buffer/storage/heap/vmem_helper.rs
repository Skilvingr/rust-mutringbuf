#![cfg_attr(doc, doc(cfg(feature = "vmem")))]
#![cfg(any(doc, feature = "vmem"))]

//! Utilities for `vmem` optimisation.

use crate::UnsafeSyncCell;
use core::ptr;
use libc::c_int;

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

#[cfg(any(target_os = "macos", target_os = "ios"))]
unsafe fn open_fd() -> c_int {
    use alloc::ffi::CString;
    use alloc::format;
    use libc::rand;

    let group_name = env!(
    "IOS_APP_GROUP_NAME",
    "In order to use feature vmem in ios, the env variable IOS_APP_GROUP_NAME must be filled with the app's group name."
    );

    libc::srand(libc::time(ptr::null_mut()) as libc::c_uint);
    let mut name;

    let fd = loop {
        name = CString::new(format!("{}{}{}", group_name, "/mrb", rand() % 99)).unwrap();
        let fd = libc::shm_open(
            name.as_ptr(),
            libc::O_CREAT | libc::O_RDWR | libc::O_EXCL,
            0700
        );

        if fd != -1 || *libc::__error() != libc::EEXIST {
            break fd;
        }
    };

    //assert_eq!(libc::shm_unlink(name.as_ptr()), 0, "shm_unlink failed");

    fd
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
unsafe fn open_fd() -> c_int {
    libc::memfd_create(c"/mrb".as_ptr(), 0)
}

pub(crate) fn new<T>(value: &[UnsafeSyncCell<T>]) -> *mut UnsafeSyncCell<T> {
    let page_size = page_size();
    assert_eq!(value.len() % page_size, 0, "len must be a multiple of page size, which is: {}.", page_size);

    unsafe {
        let size = size_of_val(value);

        let fd = open_fd();

        assert_ne!(fd, -1, "shared fd creation failed");

        if libc::ftruncate(fd, size as libc::off_t) == -1 {
            assert_eq!(libc::close(fd), 0, "close failed");
            panic!("ftruncate failed");
        }

        let buffer = libc::mmap(
            ptr::null_mut(),
            2 * size as libc::size_t,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE,
            fd, 0
        );

        panic!("AAAAAAAAAAAAAAAAAAAAAAAA: {}", *libc::__error());

        libc::mmap(
            buffer.byte_add(size),
            size as libc::size_t,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_FIXED,
            fd, 0
        );

        assert_eq!(libc::close(fd), 0, "close failed");

        let r = buffer as *mut UnsafeSyncCell<T>;
        //libc::memcpy(value.as_ptr() as _, r as _, size_of_val(value));

        r
    }
}