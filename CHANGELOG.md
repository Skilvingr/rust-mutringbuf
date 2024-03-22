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
