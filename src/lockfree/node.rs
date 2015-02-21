use atomic::TaggedPtr;

/// A linked node.
pub struct Node<T> {
  pub value: Option<T>,
  pub next: TaggedPtr<Node<T>>
}
