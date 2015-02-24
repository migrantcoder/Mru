/// Copyright (c) Migrant Coder, 2015
/// All rights reserved.

use atomic::TaggedPtr;

/// A linked node.
pub struct Node<T> {
  pub value: Option<T>,
  pub next: TaggedPtr<Node<T>>
}
