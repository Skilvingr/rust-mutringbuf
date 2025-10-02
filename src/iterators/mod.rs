//! Module containing sync and async iterators.

mod iterator_trait;
pub(crate) mod sync_iterators;
pub mod async_iterators;

#[cfg(any(feature = "async", doc))]
pub use async_iterators::{
    cons_iter::AsyncConsIter,
    detached::AsyncDetached,
    prod_iter::AsyncProdIter,
    work_iter::AsyncWorkIter
};

pub use sync_iterators::{
    cons_iter::ConsIter,
    detached::Detached,
    prod_iter::ProdIter,
    work_iter::WorkIter
};

use core::ptr;
pub use iterator_trait::MRBIterator;
pub use iterator_trait::MutableSlice;
pub use iterator_trait::NonMutableSlice;

pub(crate) use iterator_trait::iter_macros::*;

#[inline]
pub(crate) fn copy_from_slice_unchecked<T: Copy>(src: &[T], dst: &mut [T]) {
    unsafe {
        ptr::copy_nonoverlapping(
            src.as_ptr(), dst.as_mut_ptr(), src.len()
        );
    }
}

pub(crate) mod util_macros {
    macro_rules! muncher {
        (,) => { , };
        (&mut $T:ident $(,$tail:tt)*) => { &mut $T, muncher!($($tail)*) };
        (&$T:ident $(,$tail:tt)*) => { &$T, muncher!($($tail)*) };
        (&mut $T:tt $(,$tail:tt)*) => { &mut $T, muncher!($($tail)*) };
        (&$T:tt $(,$tail:tt)*) => { &$T, muncher!($($tail)*) };
        ($T:ty) => { $T };
        ($T:tt<$($gen: tt)*> $(,$tail:tt)*) => { $T<$($gen)*>, muncher!($($tail)*) };
    }

    macro_rules! delegate {
        ($Inner: tt $(($inline: tt))?, $v: vis fn $fn_name: ident (&self$(, $arg: ident $(: $arg_t: ty)?)*)
        $(-> $($ret_g: tt)*)?) => {
            #[doc = concat!("Same as [`", stringify!($Inner), "::", stringify!($fn_name), "`].")]
            $(#[$inline])?
            $v fn $fn_name(&self$(, $arg $(: $arg_t)?)*) $(-> muncher!{ $($ret_g)* })? {
                self.inner().$fn_name($($arg)*)
            }
        };
        ($Inner: tt $(($inline: tt))?, $v: vis unsafe fn $fn_name: ident (&self$(, $arg: ident $(: $arg_t: ty)?)*)
        $(-> $($ret_g: tt)*)?) => {
            #[doc = concat!("Same as [`", stringify!($Inner), "::", stringify!($fn_name), "`].")]
            /// # Safety
            #[doc = concat!("Same as [`", stringify!($Inner), "::", stringify!($fn_name), "`].")]
            $(#[$inline])?
            $v unsafe fn $fn_name(&self$(, $arg $(: $arg_t)?)*) $(-> muncher!{ $($ret_g)* })? {
                self.inner().$fn_name($($arg)*)
            }
        };
        
        ($Inner: tt $(($inline: tt))?, $v: vis fn $fn_name: ident (&$(($m: tt))? self $(, $arg: ident $(: $arg_t: ty)?)*)
        $(-> $($ret_g: tt)*)?) => {
            #[doc = concat!("Same as [`", stringify!($Inner), "::", stringify!($fn_name), "`].")]
            $(#[$inline])?
            $v fn $fn_name(&$($m)? self$(, $arg $(: $arg_t)?)*) $(-> muncher!{ $($ret_g)* })? {
                self.inner_mut().$fn_name($($arg)*)
            }
        };
        ($Inner: tt $(($inline: tt))?, $v: vis unsafe fn $fn_name: ident (&$(($m: tt))? self $(, $arg: ident $(: $arg_t: ty)?)*)
        $(-> $($ret_g: tt)*)?) => {
            #[doc = concat!("Same as [`", stringify!($Inner), "::", stringify!($fn_name), "`].")]
            /// # Safety
            #[doc = concat!("Same as [`", stringify!($Inner), "::", stringify!($fn_name), "`].")]
            $(#[$inline])?
            $v unsafe fn $fn_name(&$($m)? self$(, $arg $(: $arg_t)?)*) $(-> muncher!{ $($ret_g)* })? {
                unsafe { self.inner_mut().$fn_name($($arg)*) }
            }
        };
    }

    pub(crate) use {delegate, muncher};
}