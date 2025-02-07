<a name="v0.4.2"></a>
## v0.4.2 (07/02/2025)

Re-export trait `ConcurrentRB`.

<a name="v0.4.1"></a>
## v0.4.1 (27/01/2025)

* method `set_index` has been removed from common trait and will be available only for `Detached` iterators.
* Performance optimisations.
* Fix a memory leak.
* Re-export `Storage` trait.

<a name="v0.4.0"></a>
## v0.4.0 (15/11/2024)

* split methods are now part of traits: `HeapSplit` and `StackSplit`.
* `pop()` in `ConsIter` acts now as `ptr::read`, copying bitwise the content of the cell.
  Old `pop()` method is now called `pop_move()`.
* Some methods that were previously available only for `WorkIter`, namely:
  * `get_workable`
  * `get_workable_slice_exact`
  * `get_workable_slice_avail`
  * `get_workable_slice_multiple_of`

  can now be used by all the iterators, sync and async.
* All kinds of iterators can now be detached, yielding a `Detached` or an `AsyncDetached`.
* Async iterators can now be used in a no-std environment.
* Some imports have changed.

<a name="v0.3.1"></a>
## v0.3.1 (27/06/2024)

* Every iterator can now retrieve indices of other iterators.
* Prod iterators have now a method to fetch a tuple of mutable slices; useful to directly
  write data without the need to use copy or clone methods.
* A convenience method to busy-wait for a certain amount of items has been added.
* Fixed a bug that could cause indices to overlap.

<a name="v0.3.0"></a>
## v0.3.0 (13/05/2024)

* `ConcurrentStackRB` and `LocalStackRB` now have a new `new_zeroed()` method, which creates a new buffer
  with zeroed (uninitialised) elements.
* `new` methods in `ConcurrentHeapRB` and `LocalHeapRB` are now called `new_zeroed`.
* Methods in `ProdIter` have been split into `normal` and `*_init` ones, this in order to make possible
  to work with uninitialised memory.
* Some UBs have been fixed.
* Solve memory leaks when dropping a buffer.
* Async support has been added.

<a name="v0.2.0"></a>
## v0.2.0 (06/05/2024)

* Accumulator has been removed from `WorkIter`. There are better ways to achieve the same behaviour, like:
  [this one](https://github.com/Skilvingr/rust-mutringbuf/commit/c931aecc775fe0b222db9ff0cc4bb9ab04881bd4#diff-0b0e4efcf55f384696cdccec18c30a9dee3e81722afeca2b0509e12dc44a946b).
* Types have been simplified, so instead of e.g. `ProdIter<ConcurrentHeapRB<usize>, usize>`, one can directly write `ProdIter<ConcurrentHeapRB<usize>>`.

<a name="v0.1.3"></a>
## v0.1.3 (29/03/2024)

* `ProdIter` has now two methods to write data using a mutable reference or a mutable pointer:
  - `get_next_item_mut`;
  - `get_next_item_mut_init`.

<a name="v0.1.2"></a>
## v0.1.2 (22/03/2024)

* rename `Iterator` to `MRBIterator` to avoid annoying conflicts with other imports.

<a name="v0.1.1"></a>
## v0.1.1 (17/03/2024)

* Remove alloc useless use.
* Remove useless drop for stack-allocated buffers.
* Add tests for stack-allocated buffers.

<a name="v0.1.0"></a>
## v0.1.0 (17/03/2024)

* Rename `new_heap(capacity)` method to `new(capacity)`.
* Add a new `default method` for heap-allocated buffers.

<a name="v0.1.0-alpha.1"></a>
## v0.1.0-alpha.1 (15/03/2024)

* Rename `Iterable` trait to `Iterator`.
* Add a new trait `MutRB` to represent a generic ring buffer.
