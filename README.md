# MutRingBuf

[![crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![Rust + Miri][tests-badge]][tests-url]

[crates-badge]: https://img.shields.io/crates/v/mutringbuf.svg
[crates-url]: https://crates.io/crates/mutringbuf
[docs-badge]: https://docs.rs/mutringbuf/badge.svg
[docs-url]: https://docs.rs/mutringbuf
[tests-badge]: https://github.com/Skilvingr/rust-mutringbuf/actions/workflows/rust.yml/badge.svg
[tests-url]: https://github.com/Skilvingr/rust-mutringbuf/actions/workflows/rust.yml

A simple lock-free SPSC FIFO ring buffer, with in-place mutability.

## Should I use it?

If you are in search of a ring buffer to use in production environment, take a look at one of these, before returning here:
* [ringbuf](https://github.com/agerasev/ringbuf);
* [thingbuf](https://github.com/hawkw/thingbuf).

If you find any mistakes with this project, please, open an [issue](https://github.com/Skilvingr/rust-mutringbuf/issues/new/choose); I'll be glad to take a look!

## Performance

According to benchmarks, `ringbuf` should be a little bit faster than this crate, when executing certain operations.

On the other hand, according to tests I've made by myself using Instants, `mutringbuf` seems to be slightly faster.

I frankly don't know why, so my suggestion is to try both and decide, bearing in mind that, for typical producer-consumer use, `ringbuf` is certainly more stable and mature than this crate.

## What is the purpose of this crate?
I've written this crate to perform real-time computing over audio streams,
you can find a (simple) meaningful example [here](https://github.com/Skilvingr/rust-mutringbuf/blob/master/examples/cpal.rs).
To run it, jump [here](#tests-benchmarks-and-examples).

## Features
- `default`: `alloc`
- `alloc`: uses alloc crate, enabling heap-allocated buffers
- `async`: enables async/await support

## Usage

### A note about uninitialised items
This buffer can handle uninitialised items.
They are produced either when the buffer is created with `new_zeroed` methods, or when an initialised item
is moved out of the buffer via [`ConsIter::pop`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/sync_iterators/cons_iter/struct.ConsIter.html#method.pop) or
[`AsyncConsIter::pop`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/async_iterators/cons_iter/struct.AsyncConsIter.html#method.pop).

As also stated in `ProdIter` doc page, there are two ways to push an item into the buffer:
* normal methods can be used *only* when the location in which we are pushing the item is initialised;
* `*_init` methods must be used when that location is not initialised.

That's because normal methods implicitly drop the old value, which is a good thing if it is initialised, but
it becomes a terrible one if it's not. To be more precise, dropping an uninitialised value results in UB,
mostly in a SIGSEGV.

### Initialisation of buffer and iterators
First, a buffer has to be created.

Local buffers should be faster, due to the use of plain integers as indices, but can't obviously be used in a concurrent environment.

#### Stack-allocated buffers

```rust
use mutringbuf::{ConcurrentStackRB, LocalStackRB};
// buffers filled with default values
let concurrent_buf = ConcurrentStackRB::<usize, 10>::default();
let local_buf = LocalStackRB::<usize, 10>::default();
// buffers built from existing arrays
let concurrent_buf = ConcurrentStackRB::from([0; 10]);
let local_buf = LocalStackRB::from([0; 10]);
// buffers with uninitialised (zeroed) items
unsafe {
    let concurrent_buf = ConcurrentStackRB::<usize, 10>::new_zeroed();
    let local_buf = LocalStackRB::<usize, 10>::new_zeroed();
}
```

#### Heap-allocated buffer

```rust
use mutringbuf::{ConcurrentHeapRB, LocalHeapRB};
// buffers filled with default values
let concurrent_buf: ConcurrentHeapRB<usize> = ConcurrentHeapRB::default(10);
let local_buf: LocalHeapRB<usize> = LocalHeapRB::default(10);
// buffers built from existing vec
let concurrent_buf = ConcurrentHeapRB::from(vec![0; 10]);
let local_buf = LocalHeapRB::from(vec![0; 10]);
// buffers with uninitialised (zeroed) items
unsafe {
    let concurrent_buf: ConcurrentHeapRB <usize> = ConcurrentHeapRB::new_zeroed(10);
    let local_buf: LocalHeapRB <usize> = LocalHeapRB::new_zeroed(10);
}
```

<br/>

<div class="warning">
Please, note that the buffer uses a location to synchronise the iterators.

Thus, a buffer of size `SIZE` can keep a max amount of `SIZE - 1` values!
</div>

Then such buffer can be used in two ways:

##### Sync immutable
The normal way to make use of a ring buffer: a producer inserts values that will eventually be taken
by a consumer.

```rust
use mutringbuf::{LocalHeapRB, HeapSplit};
let buf = LocalHeapRB::from(vec![0; 10]);
let (mut prod, mut cons) = buf.split();
```

#### Sync mutable
As in the immutable case, but a third iterator `work` stands between `prod` and `cons`.

This iterator mutates elements in place.

```rust
use mutringbuf::{LocalHeapRB, HeapSplit};
let buf = LocalHeapRB::from(vec![0; 10]);
let (mut prod, mut work, mut cons) = buf.split_mut();
```

#### Async immutable
```rust ignore
use mutringbuf::LocalHeapRB;
let buf = LocalHeapRB::from(vec![0; 10]);
let (mut as_prod, mut as_cons) = buf.split_async();
```

#### Async mutable
```rust ignore
use mutringbuf::LocalHeapRB;
let buf = LocalHeapRB::from(vec![0; 10]);
let (mut as_prod, mut as_work, mut as_cons) = buf.split_mut_async();
```

Worker iterators can also be wrapped in a [`DetachedWorkIter`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/sync_iterators/detached_work_iter/struct.DetachedWorkIter.html),
as well as in an [`AsyncDetachedWorkIter`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/async_iterators/detached_work_iter/struct.AsyncDetachedWorkIter.html), indirectly pausing the consumer, in
order to explore produced data back and forth.

<br/>

Each iterator can then be passed to a thread to do its job. More information can be found
in the relative pages:
- [`ProdIter`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/sync_iterators/prod_iter/struct.ProdIter.html)
- [`WorkIter`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/sync_iterators/work_iter/struct.WorkIter.html)
- [`DetachedWorkIter`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/sync_iterators/detached_work_iter/struct.DetachedWorkIter.html)
- [`ConsIter`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/sync_iterators/cons_iter/struct.ConsIter.html)

- [`AsyncProdIter`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/async_iterators/prod_iter/struct.AsyncProdIter.html)
- [`AsyncWorkIter`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/async_iterators/work_iter/struct.AsyncWorkIter.html)
- [`AsyncDetachedWorkIter`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/async_iterators/detached_work_iter/struct.AsyncDetachedWorkIter.html)
- [`AsyncConsIter`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/async_iterators/cons_iter/struct.AsyncConsIter.html)

Note that a buffer, no matter its type, lives until the last of the iterators does so.

## Tests, benchmarks and examples
Miri test can be found within `script`.

The following commands must be run starting from the root of the crate.

Tests can be run with:

```shell
cargo test
```

Benchmarks can be run with:

```shell
RUSTFLAGS="--cfg bench" cargo bench
```

CPAL example can be run with:

```shell
RUSTFLAGS="--cfg cpal" cargo run --example cpal
```

Async example can be run with:

```shell
cargo run --example simple_async --features async
```

Every other `example_name` can be run with:
```shell
cargo run --example `example_name`
```
