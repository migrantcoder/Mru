use atomic::TaggedPtr;
use lockfree::node::Node;

/// A lock-free (non-blocking) multi-producer, multi-consumer, intrusive stack.
pub struct IntrusiveStack<T> {
  head: TaggedPtr<Node<T>>
}

impl<T> IntrusiveStack<T> {
  pub fn new() -> IntrusiveStack<T> {
    IntrusiveStack { head: TaggedPtr::nil() }
  }

  pub fn push(&mut self, mut node: TaggedPtr<Node<T>>) {
    loop {
      let head = self.head.clone();
      (*node).next.set(head.value());
      node.increment_tag();
      if self.head.compare_and_set(&head, &node) {
        break;
      }
    }
  }

  pub fn pop(&mut self) -> Option<TaggedPtr<Node<T>>> {
    loop {
      let head = self.head.clone();
      if head.is_nil() {
        return None
      }
      let mut next = (*head).next.clone();
      next.increment_tag();
      if self.head.compare_and_set(&head, &next) {
        return Some(head)
      }
    }
  }

  pub fn is_empty(&self) -> bool {
    self.head.is_nil()
  }
}
