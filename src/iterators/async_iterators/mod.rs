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
            pub struct $name<'buf, $($lt),*, $($G $(:$gen $(<Item = $IT>)?)?),*, $(const $CG: $CT),*> {
                iter: $iter,
                _item: $item
            }

            impl<'buf, $($lt),*, $($G $(:$gen $(<Item = $IT>)?)?),*, $(const $CG: $CT),*> Unpin for $name<'buf, $($lt),*, $($G),*, $($CG),*> {}

            impl<'buf, $($lt),*, $($G $(:$gen $(<Item = $IT>)?)?),*, $(const $CG: $CT),*> Future for $name<'buf, $($lt),*, $($G),*, $($CG),*> {
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

    macro_rules! gen_common_futs {
        (
            $iter: ty $(,(const $CG: ident : $CT: ty))*
        ) => {
            use crate::iterators::iterator_trait::WorkableSlice;
            
            gen_fut!{
                GetWorkableFuture<'a, B: MutRB<Item = T>, T: 'a, $((const $CG: $CT)),*>,
                $iter,
                (),
                Option<&'a mut T>,
                self {
                    let result = self.iter.inner.get_workable();
            
                    if let Some(res) = result {
                        break Poll::Ready(Some(res));
                    }
                }
            }

            gen_fut!{
                GetWorkableSliceExactFuture<'a, B: MutRB<Item = T>, T: 'a, $((const $CG: $CT)),*>,
                $iter,
                usize,
                Option<WorkableSlice<'a, T>>,
                self {
                    let count = self._item;
                    let result = self.iter.inner.get_workable_slice_exact(count);
            
                    if let Some(res) = result {
                        break Poll::Ready(Some(res));
                    }
                }
            }
            
            gen_fut!{
                GetWorkableSliceAvailFuture<'a, B: MutRB<Item = T>, T: 'a, $((const $CG: $CT)),*>,
                $iter,
                (),
                Option<WorkableSlice<'a, T>>,
                self {
                    let result = self.iter.inner.get_workable_slice_avail();
            
                    if let Some(res) = result {
                        break Poll::Ready(Some(res));
                    }
                }
            }
            
            gen_fut!{
                GetWorkableSliceMultipleOfFuture<'a, B: MutRB<Item = T>, T: 'a, $((const $CG: $CT)),*>,
                $iter,
                usize,
                Option<WorkableSlice<'a, T>>,
                self {
                    let count = self._item;
                    let result = self.iter.inner.get_workable_slice_multiple_of(count);
            
                    if let Some(res) = result {
                        break Poll::Ready(Some(res));
                    }
                }
            }
        }
    }

    pub(crate) use { gen_fut, gen_common_futs, futures_import, waker_registerer };
}