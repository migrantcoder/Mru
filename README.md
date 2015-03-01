Minimalist Rust utility libraries, including lock free data structures.

* **mru::lockfree::Queue** A lock-free multi-producer, multi-consumer unbounded LIFO queue.
* **mru::lockfree::Stack** A concurrent lock-free unbounded FIFO queue.
* **mru::lockfree::IntrusiveStack** A concurrent, lock-free, instrusive stack.
* **mru::atomic::TaggedPtr** A tagged pointer implementation suitable for implementing ABA protection for concurrent lock-free data structures.

Mru supports x86_64. The library is built and tested on OS X.

See LICENSE for licensing conditions.
