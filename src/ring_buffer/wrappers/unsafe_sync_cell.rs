use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

pub struct UnsafeSyncCell<T>(UnsafeCell<MaybeUninit<T>>);

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

impl<T> UnsafeSyncCell<T> {
    /// Constructs a new instance of `UnsafeSyncCell` which will wrap the specified value.
    #[inline]
    pub(crate) fn new(value: T) -> Self {
        Self(UnsafeCell::new(MaybeUninit::new(value)))
    }

    #[inline]
    pub(crate) unsafe fn take_inner(&self) -> T {
        let mut v = MaybeUninit::<T>::uninit();

        core::ptr::swap(&mut v, self.0.get());

        v.assume_init()
    }

    /// Unwraps the value, consuming the cell.
    /// # Safety
    /// Inner value must be initialised.
    #[inline]
    pub unsafe fn into_inner(self) -> T {
        self.0.into_inner().assume_init()
    }

    /// Returns a reference to inner value.
    /// # Safety
    /// Inner value must be initialised.
    #[inline]
    pub unsafe fn inner_ref(&self) -> &T {
        (*self.0.get()).assume_init_ref()
    }

    /// Returns a mutable reference to inner value.
    /// # Safety
    /// Not to be used to initialise inner value! Use `as_mut_ptr`, instead.
    #[inline]
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn inner_ref_mut(&self) -> &mut T {
        (*self.0.get()).assume_init_mut()
    }

    #[inline]
    pub unsafe fn as_mut_ptr(&self) -> *mut T {
        (*self.0.get()).as_mut_ptr()
    }
}
