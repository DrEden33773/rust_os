use super::*;
use core::marker::PhantomData;

impl<T: Default> FromIterator<T> for LinkedList<T> {
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let mut list = Self::new();
    iter.into_iter().for_each(|item| {
      list.push_back(item);
    });
    list
  }
}

pub struct Iter<'a, T: Default> {
  next: Option<NonNull<ListNode<T>>>,
  prev: Option<NonNull<ListNode<T>>>,
  head: NonNull<ListNode<T>>,
  tail: NonNull<ListNode<T>>,
  marker: PhantomData<&'a ListNode<T>>,
}

impl<T: Default> DoubleEndedIterator for Iter<'_, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    let curr: NonNull<ListNode<T>> = self.prev?;
    if curr.as_ptr() == self.head.as_ptr() {
      self.prev = None;
      return None;
    }
    let curr = unsafe { curr.as_ref() };
    self.prev = curr.prev;
    Some(&curr.value)
  }
}

impl<'a, T: Default> Iterator for Iter<'a, T> {
  type Item = &'a T;

  fn next(&mut self) -> Option<Self::Item> {
    let curr = self.next?;
    if curr.as_ptr() == self.tail.as_ptr() {
      self.next = None;
      return None;
    }
    let curr = unsafe { curr.as_ref() };
    self.next = curr.next;
    Some(&curr.value)
  }
}

impl<'a, T: Default> IntoIterator for &'a LinkedList<T> {
  type Item = &'a T;
  type IntoIter = Iter<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    Iter {
      next: unsafe { self.head.as_ref() }.next,
      prev: unsafe { self.tail.as_ref() }.prev,
      head: self.head,
      tail: self.tail,
      marker: PhantomData,
    }
  }
}

pub struct IterMut<'a, T: Default> {
  next: Option<NonNull<ListNode<T>>>,
  prev: Option<NonNull<ListNode<T>>>,
  head: NonNull<ListNode<T>>,
  tail: NonNull<ListNode<T>>,
  marker: PhantomData<&'a mut ListNode<T>>,
}

impl<T: Default> DoubleEndedIterator for IterMut<'_, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    let mut curr = self.prev?;
    if curr.as_ptr() == self.head.as_ptr() {
      self.prev = None;
      return None;
    }
    let curr = unsafe { curr.as_mut() };
    self.prev = curr.prev;
    Some(&mut curr.value)
  }
}

impl<'a, T: Default> Iterator for IterMut<'a, T> {
  type Item = &'a mut T;

  fn next(&mut self) -> Option<Self::Item> {
    let mut curr = self.next?;
    if curr.as_ptr() == self.tail.as_ptr() {
      self.next = None;
      return None;
    }
    let curr = unsafe { curr.as_mut() };
    self.next = curr.next;
    Some(&mut curr.value)
  }
}

impl<'a, T: Default> IntoIterator for &'a mut LinkedList<T> {
  type Item = &'a mut T;
  type IntoIter = IterMut<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    IterMut {
      next: unsafe { self.head.as_ref() }.next,
      prev: unsafe { self.tail.as_ref() }.prev,
      head: self.head,
      tail: self.tail,
      marker: PhantomData,
    }
  }
}

impl<T: Default> LinkedList<T> {
  pub fn iter(&self) -> Iter<T> {
    self.into_iter()
  }

  pub fn iter_mut(&mut self) -> IterMut<T> {
    self.into_iter()
  }
}
