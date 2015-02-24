Minimalist Rust utility libraries, including lock free data structures.

lockfree::Queue          A lock-free multi-producer, multi-consumer unbounded
                         LIFO queue.

lockfree::Stack          A concurrent lock-free unbounded FIFO queue.

lockfree::IntrusiveStack A concurrent, lock-free, instrusive stack.

atomic::TaggedPtr        A tagged pointer implementation suitable for
                         implementing ABA protection for concurrent lock-free
                         data structures.

Mru supports x86_64. The library is built and tested on OS X.

See LICENSE for licensing conditions.
