mod iterator_trait;
pub mod sync_iterators;
pub mod async_iterators;

pub use iterator_trait::MRBIterator;

pub(crate) use iterator_trait::iter_macros::*;

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
        ($Inner: tt $(($inline: tt))?, $v: vis fn $fn_name: ident (&self $(, $arg: ident $(: $arg_t: ty)?)*)
        $(-> $($ret_g: tt)*)?) => {
            #[doc = concat!("Same as [`", stringify!($Inner), "::", stringify!($fn_name), "`].")]
            $(#[$inline])?
            $v fn $fn_name(&self $(, $arg $(: $arg_t)?)*) $(-> muncher!{ $($ret_g)* })? {
                self.inner().$fn_name($($arg)*)
            }
        };
        ($Inner: tt $(($inline: tt))?, $v: vis unsafe fn $fn_name: ident (&self $(, $arg: ident $(: $arg_t: ty)?)*)
        $(-> $($ret_g: tt)*)?) => {
            #[doc = concat!("Same as [`", stringify!($Inner), "::", stringify!($fn_name), "`].")]
            /// # Safety
            #[doc = concat!("Same as [`", stringify!($Inner), "::", stringify!($fn_name), "`].")]
            $(#[$inline])?
            $v unsafe fn $fn_name(&self $(, $arg $(: $arg_t)?)*) $(-> muncher!{ $($ret_g)* })? {
                self.inner().$fn_name($($arg)*)
            }
        };
        
        ($Inner: tt $(($inline: tt))?, $v: vis fn $fn_name: ident (& $(($m: tt))? self $(, $arg: ident $(: $arg_t: ty)?)*)
        $(-> $($ret_g: tt)*)?) => {
            #[doc = concat!("Same as [`", stringify!($Inner), "::", stringify!($fn_name), "`].")]
            $(#[$inline])?
            $v fn $fn_name(& $($m)? self $(, $arg $(: $arg_t)?)*) $(-> muncher!{ $($ret_g)* })? {
                self.inner_mut().$fn_name($($arg)*)
            }
        };
        ($Inner: tt $(($inline: tt))?, $v: vis unsafe fn $fn_name: ident (& $(($m: tt))? self $(, $arg: ident $(: $arg_t: ty)?)*)
        $(-> $($ret_g: tt)*)?) => {
            #[doc = concat!("Same as [`", stringify!($Inner), "::", stringify!($fn_name), "`].")]
            /// # Safety
            #[doc = concat!("Same as [`", stringify!($Inner), "::", stringify!($fn_name), "`].")]
            $(#[$inline])?
            $v unsafe fn $fn_name(& $($m)? self $(, $arg $(: $arg_t)?)*) $(-> muncher!{ $($ret_g)* })? {
                self.inner_mut().$fn_name($($arg)*)
            }
        };
    }

    pub(crate) use {delegate, muncher};
}