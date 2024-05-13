#![cfg(any(feature = "async", doc))]

pub mod prod_iter;
pub mod work_iter;
pub mod cons_iter;
pub mod detached_work_iter;

pub(crate) mod async_macros {
    macro_rules! futures_import { () => {
        use core::future::Future;
        use core::pin::Pin;
        use core::task::{Context, Poll, Waker};
    }}

    macro_rules! waker_registerer {
        () => {
            fn register_waker(&mut self, waker: &Waker) {
                self.waker.get_or_insert(waker.clone()).clone_from(waker);
            }
        };
    }

    macro_rules! gen_fut {
        (
            $name:ident < $($lt:lifetime),*, $($G:ident $(: $gen:tt $(<Item = $IT: tt>)?)? $(,)?)* $((const $CG: ident : $CT: ty))*>, // name of the struct, including generics
            $iter: ty, // type of the internal iterator
            $item: ty, // type of the externally passed item
            $output: ty, // output of the future
            $self: ident $f: block // block of code to be executed inside `poll()`
        ) => {
            pub struct $name<$($lt),*, $($G $(:$gen $(<Item = $IT>)?)?),*, $(const $CG: $CT),*> {
                iter: $iter,
                _item: $item
            }

            impl<$($lt),*, $($G $(:$gen $(<Item = $IT>)?)?),*, $(const $CG: $CT),*> Unpin for $name<$($lt),*, $($G),*, $($CG),*> {}

            impl<$($lt),*, $($G $(:$gen $(<Item = $IT>)?)?),*, $(const $CG: $CT),*> Future for $name<$($lt),*, $($G),*, $($CG),*> {
                type Output = $output;

                fn poll(mut $self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {

                    let mut waker_registered = false;
                    loop {
                        $f

                        if waker_registered {
                            break Poll::Pending;
                        }
                        $self.iter.register_waker(cx.waker());
                        waker_registered = true;
                    }
                }
            }
        }
    }

    pub(crate) use { gen_fut, futures_import, waker_registerer };
}