/// Copyright (c) Migrant Coder, 2015
/// All rights reserved.

use std::mem;
use std::sync::Arc;

use atomic::TaggedPtr;
use lockfree::intrusivestack::IntrusiveStack;
use lockfree::node::Node;

/// A lock-free (non-blocking) concurrent stack.
pub struct Stack<T: Clone> {
  free: IntrusiveStack<T>,
  stack: IntrusiveStack<T>
}

/// A thread-safe producer for pushing onto a stack.
pub struct StackProducer<T: Clone> {
  stack: Arc<Stack<T>>
}

/// A thread-safe consumer for popping off of a stack.
pub struct StackConsumer<T: Clone> {
  stack: Arc<Stack<T>>
}

impl<T: Clone> Stack<T> {
  pub fn new(initial_capacity: usize) -> Stack<T> {
    let mut stack: Stack<T> =
        Stack { free: IntrusiveStack::new(), stack: IntrusiveStack::new() };
    stack.extend(initial_capacity);
    stack
  }

  fn new_free_node() -> TaggedPtr<Node<T>> {
    let n: Box<Node<T>> = Box::new(Node { value: None, next: TaggedPtr::nil() });
    TaggedPtr::from_box(n)
  }

  pub fn is_empty(&self) -> bool {
    self.stack.is_empty()
  }

  pub fn push(&mut self, value: T) {
    let mut node =
        match self.free.pop() {
          Some(n) => n,
          None => Stack::new_free_node()
        };
    (*node).value = Some(value);
    self.stack.push(node);
  }

  pub fn pop(&mut self) -> Option<T> {
    match self.stack.pop() {
      Some(ptr) => {
        let value = (*ptr).value.clone();
        self.free.push(ptr);
        value
      },
      None => None
    }
  }

  /// Release capacity not currently being used.
  pub fn compact(&mut self) {
    while !self.free.is_empty() {
      self.free.pop();
    }
  }

  /// Extend capacity by the specified count.
  pub fn extend(&mut self, count: usize) {
    for _ in 0..count {
      self.free.push(Stack::new_free_node());
    }
  }
}

#[unsafe_destructor]
impl<T: Clone> Drop for Stack<T> {
  fn drop(&mut self) {
    self.compact();
  }
}

impl<T: Clone>  StackProducer<T> {
  pub fn new(stack: &Arc<Stack<T>>) -> StackProducer<T> {
    StackProducer { stack: stack.clone() }
  }

  pub fn push(&self, value: T) {
    let stack: &mut Stack<T> = unsafe { mem::transmute(&*self.stack) };
    stack.push(value)
  }
}

impl<T: Clone> Clone for StackProducer<T> {
  fn clone(&self) -> Self {
    StackProducer { stack: self.stack.clone() }
  }
}

unsafe impl<T> Send for StackProducer<T> {}
unsafe impl<T> Sync for StackProducer<T> {}

impl<T: Clone>  StackConsumer<T> {
  pub fn new(stack: &Arc<Stack<T>>) -> StackConsumer<T> {
    StackConsumer { stack: stack.clone() }
  }

  pub fn pop(&self) -> Option<T> {
    let stack: &mut Stack<T> = unsafe { mem::transmute(&*self.stack) };
    stack.pop()
  }
}

impl<T: Clone> Clone for StackConsumer<T> {
  fn clone(&self) -> Self {
    StackConsumer { stack: self.stack.clone() }
  }
}

unsafe impl<T> Send for StackConsumer<T> {}
unsafe impl<T> Sync for StackConsumer<T> {}
