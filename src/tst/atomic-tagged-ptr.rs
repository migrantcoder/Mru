extern crate mru;

use mru::atomic::TaggedPtr;
use mru::lockfree::{ InstrusiveStack, Node };

fn main() {
  let stack1: InstrusiveStack<usize> = InstrusiveStack::new();
  let stack2: InstrusiveStack<usize> = InstrusiveStack::new();
  let n1 =
      TaggedPtr::from_box(
          Box::new(
              Node { value: 42, next: TaggedPtr::nil() }));
  let n2 =
      TaggedPtr::from_box(
          Box::new(
              Node { value: 5, next: TaggedPtr::nil() }));
  stack1.push(n1);
  stack1.push(n2);
  {
    let p1 = stack1.pop().unwrap();
    let p2 = stack1.pop().unwrap();
    println!("{}", (*p1).value);
    println!("{}", (*p2).value);
    stack2.push(p1);
    stack2.push(p2);
  }
  {
    let p1 = stack2.pop().unwrap();
    let p2 = stack2.pop().unwrap();
    println!("{}", (*p1).value);
    println!("{}", (*p2).value);
    p1.to_box();
    p2.to_box();
  }
}
