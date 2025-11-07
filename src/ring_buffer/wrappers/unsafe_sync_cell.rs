use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr::slice_from_raw_parts;

/// Sync version of `UnsafeCell<MaybeUninit<T>>`.
/// While it should not be used outside of this crate, it may be useful in certain scenarios.
#[repr(transparent)]
pub struct UnsafeSyncCell<T>(UnsafeCell<MaybeUninit<T>>);

impl<T> Drop for UnsafeSyncCell<T> {
    fn drop(&mut self) {
        if !UnsafeSyncCell::check_zeroed(self.0.get_mut().as_mut_ptr()) {
            unsafe { self.0.get_mut().assume_init_drop() }
        }
    }
}

unsafe impl<T: Sync> Sync for UnsafeSyncCell<T> {}
impl<T: Default> Default for UnsafeSyncCell<T> {
    /// Creates an `UnsafeSyncCell`, with the `Default` value for T.
    fn default() -> UnsafeSyncCell<T> {
        UnsafeSyncCell::new(Default::default())
    }
}
impl<T> From<T> for UnsafeSyncCell<T> {
    /// Creates a new `UnsafeSyncCell<T>` containing the given value.
    fn from(t: T) -> UnsafeSyncCell<T> {
        UnsafeSyncCell::new(t)
    }
}

impl<T: Clone> Clone for UnsafeSyncCell<T> {
    fn clone(&self) -> Self {
        if Self::check_zeroed(self.as_mut_ptr() as _) {
            UnsafeSyncCell::new_zeroed()
        } else {
            unsafe { Self::from(self.inner_ref().clone()) }
        }
    }
}

impl<T> UnsafeSyncCell<T> {
    /// Constructs a new instance of `UnsafeSyncCell` which wraps the specified value.
    #[inline]
    pub(crate) fn new(value: T) -> Self {
        Self(UnsafeCell::new(MaybeUninit::new(value)))
    }

    /// Constructs a new instance of `UnsafeSyncCell` filled with zeros.
    #[inline]
    pub(crate) fn new_zeroed() -> Self {
        Self(UnsafeCell::new(MaybeUninit::zeroed()))
    }

    /// Checks whether the memory pointed to by `ptr`, for a certain type `T`, is composed entirely of zeros.
    #[inline]
    pub fn check_zeroed(ptr: *const T) -> bool {
        unsafe {
            (*slice_from_raw_parts(ptr as *const u8, size_of::<T>()))
                .iter()
                .all(|x| *x == 0)
        }
    }

    /// Takes inner value, replacing its old location with zeros.
    /// # Safety
    /// The cell will be uninitialised and so must be initialised again with [`Self::as_mut_ptr`].
    #[inline]
    pub unsafe fn take_inner(&self) -> T {
        unsafe { core::mem::replace(&mut *self.0.get(), MaybeUninit::<T>::zeroed()).assume_init() }
    }

    /// Reads and duplicates the value.
    /// For more information on `MaybeUninit`, refer to
    /// [the Rust documentation](https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#method.assume_init_read).
    /// # Safety
    /// The inner value must be initialised (i.e., properly constructed).
    #[inline]
    pub unsafe fn inner_duplicate(&self) -> T {
        unsafe { (*self.0.get()).assume_init_read() }
    }

    /// Returns a reference to inner value.
    /// # Safety
    /// Inner value must be initialised (i.e., properly constructed).
    #[inline]
    pub unsafe fn inner_ref<'a>(&self) -> &'a T {
        unsafe { (*self.0.get()).assume_init_ref() }
    }

    /// Returns a mutable reference to inner value.
    /// # Safety
    /// Not to be used to initialise inner value! Use [`Self::as_mut_ptr`], instead.
    #[inline]
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn inner_ref_mut<'a>(&self) -> &'a mut T {
        unsafe { (*self.0.get()).assume_init_mut() }
    }

    /// Gets a mutable pointer to the contained value.
    /// See [`MaybeUninit::as_mut_ptr`].
    #[inline]
    pub fn as_mut_ptr(&self) -> *mut T {
        unsafe { (*self.0.get()).as_mut_ptr() }
    }
}
