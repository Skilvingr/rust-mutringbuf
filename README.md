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

## Usage

### Initialisation of buffer and iterators
First, a buffer has to be created.

Local buffers should be faster, due to the use of plain integers as indices, but can't obviously be used in a concurrent environment.

#### Stack-allocated buffers

```rust
use mutringbuf::{ConcurrentStackRB, LocalStackRB};
let concurrent_buf = ConcurrentStackRB::<usize, 10>::default();
let local_buf = LocalStackRB::<usize, 10>::default();
```
or:
```rust
use mutringbuf::{ConcurrentStackRB, LocalStackRB};
let concurrent_buf = ConcurrentStackRB::from([0; 10]);
let local_buf = LocalStackRB::from([0; 10]);
```

#### Heap-allocated buffer

```rust
use mutringbuf::{ConcurrentHeapRB, LocalHeapRB};
let concurrent_buf: ConcurrentHeapRB<usize> = ConcurrentHeapRB::new(10);
let local_buf: LocalHeapRB<usize> = LocalHeapRB::new(10);
```
or:
```rust
use mutringbuf::{ConcurrentHeapRB, LocalHeapRB};
let concurrent_buf = ConcurrentHeapRB::from(vec![0; 10]);
let local_buf = LocalHeapRB::from(vec![0; 10]);
```

<br/>

<div class="warning">
Please, note that the buffer uses a location to synchronise the iterators.

Thus, a buffer of size `SIZE` can keep a max amount of `SIZE - 1` values!
</div>

Then such buffer can be used in two ways:

#### Immutable
The normal way to make use of a ring buffer: a producer inserts values that will eventually be taken
by a consumer.

```rust
use mutringbuf::LocalHeapRB;
let buf = LocalHeapRB::from(vec![0; 10]);
let (mut prod, mut cons) = buf.split();
```

#### Mutable
As in the immutable case, but a third iterator `work` stands between `prod` and `cons`.

This iterator mutates elements in place.

```rust
use mutringbuf::LocalHeapRB;
let buf = LocalHeapRB::from(vec![0; 10]);
let (mut prod, mut work, mut cons) = buf.split_mut();
```

Worker iterator can also be wrapped in a [`DetachedWorkIter`], indirectly pausing the consumer, in
order to explore produced data back and forth.

<br/>

Each iterator can then be passed to a thread to do its job. More information can be found
in the relative pages:
- [`ProdIter`]
- [`WorkIter`]
- [`ConsIter`]

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

Every other `example_name` can be run with:
```shell
cargo run --example `example_name`
```

## To do
- [ ] Implement an async/await version;
- [ ] (Maybe) add the ability to spawn an arbitrary number of worker iterators.
