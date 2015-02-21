use std::boxed;
use std::mem;
use std::sync::atomic::AtomicPtr;
use atomic::{ TaggedPtr };

pub fn hello() { println!("Hello from lockfree!"); }

pub struct Node<T> {
  pub value: T,
  pub next: TaggedPtr<Node<T>>
}

pub struct InstrusiveStack<T> {
  head: TaggedPtr<Node<T>>
}

pub struct Stack<T> {
  free: InstrusiveStack<T>,
  stack: InstrusiveStack<T>
}

/// A lock-free (non-blocking but not wait-free) concurrent, intrusive stack.
impl<T> InstrusiveStack<T> {
  pub fn new() -> InstrusiveStack<T> {
    InstrusiveStack { head: TaggedPtr::nil() }
  }

  pub fn push(&self, node: TaggedPtr<Node<T>>) {
    loop {
      let head = self.head.clone();
      (*node).next.set(head.value());
      if self.head.compare_and_set(&head, &node) {
        break;
      }
    }
  }

  pub fn pop(&self) -> Option<TaggedPtr<Node<T>>> {
    loop {
      let head = self.head.clone();
      if head.is_nil() {
        return None
      }
      let next = (*head).next.clone();
      next.increment_tag();
      if self.head.compare_and_set(&head, &next) {
        return Some(head)
      }
    }
  }
}
