# MutRingBuf

[![crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![Rust + Miri + Sanitisers][tests-badge]][tests-url]

[crates-badge]: https://img.shields.io/crates/v/mutringbuf.svg
[crates-url]: https://crates.io/crates/mutringbuf
[docs-badge]: https://docs.rs/mutringbuf/badge.svg
[docs-url]: https://docs.rs/mutringbuf
[tests-badge]: https://github.com/Skilvingr/rust-mutringbuf/actions/workflows/rust.yml/badge.svg
[tests-url]: https://github.com/Skilvingr/rust-mutringbuf/actions/workflows/rust.yml

A lock-free single-producer, single-consumer (SPSC) ring buffer with in-place mutability, asynchronous support,
and virtual memory optimisation.

## Performance

Benchmarks indicate that [ringbuf](https://github.com/agerasev/ringbuf) may outperform this crate in certain operations.
However, my own tests using `Instant` suggest that `mutringbuf` is slightly faster. I recommend trying both to see which
one meets your needs better.

## Purpose

This crate was developed for real-time audio stream processing. You can find a simple example
[here](https://github.com/Skilvingr/rust-mutringbuf/blob/master/examples/cpal.rs). For instructions on running it, jump
to the [Tests, Benchmarks, and Examples](#tests-benchmarks-and-examples) section.

## Features

- `default`: Enables the `alloc` feature.
- `alloc`: Uses the `alloc` crate for heap-allocated buffers.
- `async`: Provides support for async/await.
- `vmem`: Enables virtual memory optimisations.

## `vmem` Extension

An interesting optimisation for circular buffers involves mapping the underlying buffer to two contiguous regions of
virtual memory. More information can be found [here](https://en.wikipedia.org/wiki/Circular_buffer#Optimization).

This crate supports this optimisation through the `vmem` feature, which can only be used with heap-allocated buffers and
is currently limited to `unix` targets. The buffer size (length of the buffer times the size of the stored type) must be a multiple of the system's page size (usually `4096` for x86_64).
When using the `default` and `new_zeroed` methods, the correct size is calculated based on the provided minimum size.
However, when using the `from` methods, the user must ensure that this requirement is met to avoid panics.

At the moment, the feature has been tested on GNU/Linux, Android, macOS and iOS.

### A Note About iOS

`vmem` works by allocating shared memory. While this doesn't represent a problem on other platforms,
it is different on iOS.
Users should create an app group
(more information [here](https://developer.apple.com/documentation/xcode/configuring-app-groups))
and then set the environment variable `IOS_APP_GROUP_NAME` to the name of that group.

## Usage

### Note on Uninitialised Items

This buffer can handle uninitialised items, which can occur when the buffer is created with `new_zeroed` methods or when
an initialised item is moved out via [`ConsIter::pop`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/sync_iterators/cons_iter/struct.ConsIter.html#method.pop)
or [`AsyncConsIter::pop`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/async_iterators/cons_iter/struct.AsyncConsIter.html#method.pop).

As noted in the `ProdIter` documentation, there are two ways to push an item into the buffer:
- Normal methods can only be used when the target location is initialised.
- `*_init` methods must be used when the target location is uninitialised.

Using normal methods on uninitialised values can lead to undefined behaviour (UB), such as a segmentation fault (SIGSEGV).

### Initialising Buffers and Iterators

First, create a buffer. Local buffers are generally faster due to the use of plain integers as indices, but they are not
suitable for concurrent environments. In some cases, concurrent buffers may perform better than local ones.

#### Stack-Allocated Buffers

```rust
use mutringbuf::{ConcurrentStackRB, LocalStackRB, AsyncStackRB};

// Buffers filled with default values
let concurrent_buf = ConcurrentStackRB::<usize, 4096>::default();
let local_buf = LocalStackRB::<usize, 4096>::default();
let async_buf = AsyncStackRB::<usize, 4096>::default();

// Buffers built from existing arrays
let concurrent_buf = ConcurrentStackRB::from([0; 4096]);
let local_buf = LocalStackRB::from([0; 4096]);
let async_buf = AsyncStackRB::from([0; 4096]);

// Buffers with uninitialised (zeroed) items
unsafe {
    let concurrent_buf = ConcurrentStackRB::<usize, 4096>::new_zeroed();
    let local_buf = LocalStackRB::<usize, 4096>::new_zeroed();
    let async_buf = AsyncStackRB::<usize, 4096>::new_zeroed();
}
```

#### Heap-Allocated Buffers
```rust
use mutringbuf::{ConcurrentHeapRB, LocalHeapRB, AsyncHeapRB};

// Buffers filled with default values
let concurrent_buf: ConcurrentHeapRB<usize> = ConcurrentHeapRB::default(4096);
let local_buf: LocalHeapRB<usize> = LocalHeapRB::default(4096);
let async_buf: AsyncHeapRB<usize> = AsyncHeapRB::default(4096);

// Buffers built from existing vectors
let concurrent_buf = ConcurrentHeapRB::from(vec![0; 4096]);
let local_buf = LocalHeapRB::from(vec![0; 4096]);
let async_buf = AsyncHeapRB::from(vec![0; 4096]);

// Buffers with uninitialised (zeroed) items
unsafe {
    let concurrent_buf: ConcurrentHeapRB<usize> = ConcurrentHeapRB::new_zeroed(4096);
    let local_buf: LocalHeapRB<usize> = LocalHeapRB::new_zeroed(4096);
    let async_buf: AsyncHeapRB<usize> = AsyncHeapRB::new_zeroed(4096);
}
```

### Buffer Usage

The buffer can be utilised in two primary ways:

#### Immutable

This is the standard way to use a ring buffer, where a producer inserts values that will eventually be consumed.

```rust
use mutringbuf::{LocalHeapRB, HeapSplit};

let buf = LocalHeapRB::from(vec![0; 4096]);
let (mut prod, mut cons) = buf.split();
```

#### Mutable

Similar to the immutable case, but with an additional iterator `work` that allows for in-place mutation of elements.

```rust
use mutringbuf::{LocalHeapRB, HeapSplit};

let buf = LocalHeapRB::from(vec![0; 4096]);
let (mut prod, mut work, mut cons) = buf.split_mut();
```

Iterators can also be wrapped in a [`Detached`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/sync_iterators/detached/struct.Detached.html)
or an [`AsyncDetached`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/async_iterators/detached/struct.AsyncDetached.html),
allowing for exploration of produced data back and forth while indirectly pausing the consumer.

Each iterator can be passed to a thread to perform its tasks. More information can be found in the respective documentation pages:

- **Sync**
  - [`ProdIter`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/sync_iterators/prod_iter/struct.ProdIter.html)
  - [`WorkIter`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/sync_iterators/work_iter/struct.WorkIter.html)
  - [`Detached`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/sync_iterators/detached/struct.Detached.html)
  - [`ConsIter`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/sync_iterators/cons_iter/struct.ConsIter.html)

- **Async**
  - [`AsyncProdIter`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/async_iterators/prod_iter/struct.AsyncProdIter.html)
  - [`AsyncWorkIter`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/async_iterators/work_iter/struct.AsyncWorkIter.html)
  - [`AsyncDetached`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/async_iterators/detached/struct.AsyncDetached.html)
  - [`AsyncConsIter`](https://docs.rs/mutringbuf/latest/mutringbuf/iterators/async_iterators/cons_iter/struct.AsyncConsIter.html)

Note that a buffer, regardless of its type, remains alive until the last of its iterators is dropped.

## Tests, Benchmarks, and Examples

Miri tests can be found within the `script` directory. The following commands should be run from the root of the crate.

To run tests:

```shell
cargo +nightly test
```

To run benchmarks:

```shell
cargo bench
```

To run the CPAL example:

```shell
RUSTFLAGS="--cfg cpal" cargo run --example cpal
```

If you encounter an error like:
`ALSA lib pcm_dsnoop.c:567:(snd_pcm_dsnoop_open) unable to open slave`, please refer to
[this issue](https://github.com/Uberi/speech_recognition/issues/526#issuecomment-1670900376).

To run the async example:
```shell
cargo run --example simple_async --features async
```

Every other example_name can be run with:
```shell
cargo run --example `example_name`
```
