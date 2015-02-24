/// Copyright (c) Migrant Coder, 2015
/// All rights reserved.

use std::boxed;
use std::boxed::Box;
use std::mem;
use std::num::ToPrimitive;
use std::ops::{ Deref, DerefMut };
use std::sync::atomic::{ AtomicPtr, Ordering };


/// A non-owning pointer type that supports tagging and atomic compare and set.
pub struct TaggedPtr<T> {
  ptr: AtomicPtr<T>
}

impl<T> TaggedPtr<T> {
  pub fn nil() -> TaggedPtr<T> {
    unsafe { TaggedPtr {ptr: AtomicPtr::new(mem::transmute(0x0u64)) } }
  }

  /// Set to the pointer to box's object.  The box is consumed so the object
  /// must be manually destroyed or re-boxed.
  pub fn from_box(b: Box<T>) -> TaggedPtr<T> {
    unsafe { TaggedPtr {ptr: AtomicPtr::new(boxed::into_raw(b)) } }
  }

  /// Create from raw pointer.
  pub fn from_raw(ptr: *mut T) -> TaggedPtr<T> {
    TaggedPtr {ptr: AtomicPtr::new(ptr) }
  }

  /// Transfer ownership to box, clearing this.
  pub fn to_box(&self) -> Option<Box<T>> {
    if self.is_nil() {
      None
    } else {
      let value = self.get();
      unsafe { self.set(mem::transmute(0x0u64)); }
      Some(unsafe { Box::from_raw(value) })
    }
  }

  pub fn is_nil(&self) -> bool {
    unsafe { mem::transmute(self.get()) == 0u64 }
  }

  /// Set the value.
  pub fn set(&self, ptr: *mut T) {
    self.ptr.store(ptr, Ordering::Relaxed);
  }

  /// Return the untagged pointer value.
  pub fn get(&self) -> *mut T {
    unsafe {
      let ptr = self.ptr.load(Ordering::Relaxed);
      let u64ptr: u64 = mem::transmute(ptr);
      mem::transmute(u64ptr & 0x0000_ffff_ffff_ffffu64)
    }
  }

  pub fn to_u64(&self) -> u64 {
    unsafe { mem::transmute(self.value()) }
  }

  /// Return the tag value.
  pub fn get_tag(&self) -> u16 {
    unsafe {
      let ptr = self.ptr.load(Ordering::Relaxed);
      let u64ptr: u64 = mem::transmute(ptr);
      (u64ptr >> 48).to_u16().unwrap()
    }
  }

  /// Set the tag value.
  pub fn set_tag(&mut self, tag: u16) {
    unsafe {
      let u64ptr: u64 = mem::transmute(self.get());
      let tagged = u64ptr | (tag.to_u64().unwrap() << 48);
      self.ptr.store(mem::transmute(tagged), Ordering::Relaxed);
    }
  }

  /// Return the tagged pointer value.
  pub fn value(&self) -> *mut T {
    self.ptr.load(Ordering::Relaxed)
  }

  pub fn increment_tag(&mut self) {
    let tag = self.get_tag();
    self.set_tag(tag + 1);
  }

  /// Atomically and with sequentially consistent memory ordering, compare this
  /// to expected and if equal set this to desired. 
  pub fn compare_and_set(
      &mut self,
      expected: &TaggedPtr<T>,
      desired: &TaggedPtr<T>) -> bool {
    let expected_value = expected.value();
    expected_value == 
        self.ptr.compare_and_swap(
            expected_value,
            desired.value(),
            Ordering::SeqCst)
  }
}

impl<T> Eq for TaggedPtr<T> {
}

impl<T> PartialEq for TaggedPtr<T> {
  fn eq(&self, rhs: &Self) -> bool {
    self.ptr.load(Ordering::Relaxed) == rhs.ptr.load(Ordering::Relaxed)
  }
}

impl<T> Deref for TaggedPtr<T> {
  type Target = T;

  fn deref<'a>(&'a self) -> &'a T {
    unsafe { &*self.get() }
  }
}

impl<T> DerefMut for TaggedPtr<T> {
  fn deref_mut<'a>(&'a mut self) -> &'a mut T {
    unsafe { & mut *self.get() }
  }
}

impl<T> Clone for TaggedPtr<T> {
  fn clone(&self) -> Self {
    TaggedPtr::from_raw(self.value())
  }
}

unsafe impl<T> Sync for TaggedPtr<T> {}
