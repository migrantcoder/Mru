/// Copyright (c) Migrant Coder, 2015
/// All rights reserved.

use std::mem;
use std::sync::Arc;

use atomic::TaggedPtr;
use lockfree::stack::Stack;
use lockfree::node::Node;

/// A lock-free (non-blocking) multi-producer, multi-consumer queue.
pub struct Queue<T: Clone> {
  free: Stack<TaggedPtr<Node<T>>>,
  head: TaggedPtr<Node<T>>,
  tail: TaggedPtr<Node<T>>
}

/// A thread-safe producer for pushing onto a queue.
pub struct QueueProducer<T: Clone> {
  queue: Arc<Queue<T>>
}

/// A thread-safe consumer for popping off of a queue.
pub struct QueueConsumer<T: Clone> {
  queue: Arc<Queue<T>>
}

impl<T: Clone> Queue<T> {
  pub fn new(initial_capacity: usize) -> Queue<T> {
    let node = Queue::new_free_node();
    let queue = Queue {
      free: Stack::new(initial_capacity),
      head: node.clone(),
      tail: node.clone() 
    };

    assert!(queue.head == queue.tail);

    queue 
  }

  fn new_free_node() -> TaggedPtr<Node<T>> {
    let n: Box<Node<T>> = Box::new(Node { value: None, next: TaggedPtr::nil() });
    TaggedPtr::from_box(n)
  }

  pub fn is_empty(&self) -> bool {
    self.head.is_nil()
  }

  pub fn push(&mut self, value: T) {
    let mut node =
        match self.free.pop() {
          Some(n) => n,
          None => Queue::new_free_node()
        };
    (*node).value = Some(value);
    (*node).next = TaggedPtr::nil();

    let mut tail: TaggedPtr<Node<T>> = TaggedPtr::nil();

    loop {
      tail = self.tail.clone();
      let mut next = (*tail).next.clone();

      // Verify read of tail and next is consistent.
      if tail != self.tail {
        continue;
      }

      if next.is_nil() {
        // Attempt to link in the new node.
        node.set_tag(next.get_tag());
        node.increment_tag();
        if (*self.tail).next.compare_and_set(&next, &node) {
          break;
        }
      } else {
        // The tail pointer has fallen behind, attempt to move it along.
        next.set_tag(tail.get_tag());
        next.increment_tag();
        self.tail.compare_and_set(&tail, &next);
        continue;
      }
    }

    // If this update fails, the next push/pop will update the tail pointer.
    node.set_tag(tail.get_tag());
    node.increment_tag();
    self.tail.compare_and_set(&tail, &node);
  }

  pub fn pop(&mut self) -> Option<T> {
    loop {
      // Read the state in an order allowing consistency verification.
      let head = self.head.clone();
      let tail = self.tail.clone();
      let mut next = (*head).next.clone();

      // Verify read of head, tail and head.next is consistent.
      if head != self.head {
        continue;
      }

      if head == tail {
        if next.is_nil() {
          // The queue is empty.
          return None;
        } else {
          // The tail pointer has fallen behind, attempt to move it along.
          next.set_tag(tail.get_tag());
          next.increment_tag();
          self.tail.compare_and_set(&tail, &next);
        }
      }

      assert!(!next.is_nil());

      // Copy out the first node's value and dequeue it.
      let value = (*next).value.clone();
      next.set_tag(head.get_tag());
      next.increment_tag();
      if !self.head.compare_and_set(&head, &next) {
        continue;
      }

      // Free the old head node.
      self.free.push(head);

      return value;
    }
  }

  /// Release capacity not currently being used.
  pub fn compact(&mut self) {
    self.free.compact();
  }

  /// Extend capacity by the specified count.
  pub fn extend(&mut self, count: usize) {
    self.free.extend(count);
  }
}

#[unsafe_destructor]
impl<T: Clone> Drop for Queue<T> {
  fn drop(&mut self) {
    self.compact();
  }
}

impl<T: Clone>  QueueProducer<T> {
  pub fn new(queue: &Arc<Queue<T>>) -> QueueProducer<T> {
    QueueProducer { queue: queue.clone() }
  }

  pub fn push(&self, value: T) {
    let queue: &mut Queue<T> = unsafe { mem::transmute(&*self.queue) };
    queue.push(value)
  }
}

impl<T: Clone> Clone for QueueProducer<T> {
  fn clone(&self) -> Self {
    QueueProducer { queue: self.queue.clone() }
  }
}

unsafe impl<T> Send for QueueProducer<T> {}
unsafe impl<T> Sync for QueueProducer<T> {}

impl<T: Clone>  QueueConsumer<T> {
  pub fn new(queue: &Arc<Queue<T>>) -> QueueConsumer<T> {
    QueueConsumer { queue: queue.clone() }
  }

  pub fn pop(&self) -> Option<T> {
    let queue: &mut Queue<T> = unsafe { mem::transmute(&*self.queue) };
    queue.pop()
  }
}

impl<T: Clone> Clone for QueueConsumer<T> {
  fn clone(&self) -> Self {
    QueueConsumer { queue: self.queue.clone() }
  }
}

unsafe impl<T> Send for QueueConsumer<T> {}
unsafe impl<T> Sync for QueueConsumer<T> {}
