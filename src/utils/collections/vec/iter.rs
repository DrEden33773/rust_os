use super::*;
use core::marker::PhantomData;

impl<T> FromIterator<T> for Vec<T> {
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let mut vec = Self::new();
    iter.into_iter().for_each(|e| vec.push(e));
    vec
  }
}

pub struct Iter<'a, T> {
  data: NonNull<T>,
  len: usize,
  next_index: usize,
  prev_index: usize,
  is_head: bool,
  marker: PhantomData<&'a T>,
}

impl<T> DoubleEndedIterator for Iter<'_, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    if self.is_head {
      return None;
    }
    let curr_index = self.prev_index;
    let curr = unsafe { self.data.as_ptr().add(self.prev_index).as_ref() };
    if curr_index == 0 {
      self.is_head = true;
    } else {
      self.prev_index -= 1;
    }
    curr
  }
}

impl<'a, T> Iterator for Iter<'a, T> {
  type Item = &'a T;

  fn next(&mut self) -> Option<Self::Item> {
    let curr_index = self.next_index;
    if curr_index >= self.len {
      return None;
    }
    let curr = unsafe { self.data.as_ptr().add(self.next_index).as_ref() };
    self.next_index += 1;
    curr
  }
}

impl<'a, T> IntoIterator for &'a Vec<T> {
  type Item = &'a T;
  type IntoIter = Iter<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    Iter {
      data: self.data,
      len: self.len,
      next_index: 0,
      prev_index: if self.is_empty() { 0 } else { self.len - 1 },
      is_head: false,
      marker: PhantomData,
    }
  }
}

pub struct IterMut<'a, T> {
  data: NonNull<T>,
  len: usize,
  next_index: usize,
  prev_index: usize,
  is_head: bool,
  marker: PhantomData<&'a mut T>,
}

impl<T> DoubleEndedIterator for IterMut<'_, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    if self.is_head {
      return None;
    }
    let curr_index = self.prev_index;
    let curr = unsafe { self.data.as_ptr().add(self.prev_index).as_mut() };
    if curr_index == 0 {
      self.is_head = true;
    } else {
      self.prev_index -= 1;
    }
    curr
  }
}

impl<'a, T> Iterator for IterMut<'a, T> {
  type Item = &'a mut T;

  fn next(&mut self) -> Option<Self::Item> {
    let curr_index = self.next_index;
    if curr_index >= self.len {
      return None;
    }
    let curr = unsafe { self.data.as_ptr().add(self.next_index).as_mut() };
    self.next_index += 1;
    curr
  }
}

impl<'a, T> IntoIterator for &'a mut Vec<T> {
  type Item = &'a mut T;
  type IntoIter = IterMut<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    IterMut {
      data: self.data,
      len: self.len,
      next_index: 0,
      prev_index: if self.is_empty() { 0 } else { self.len - 1 },
      is_head: false,
      marker: PhantomData,
    }
  }
}

impl<T> Vec<T> {
  pub fn iter(&self) -> Iter<T> {
    self.into_iter()
  }

  pub fn iter_mut(&mut self) -> IterMut<T> {
    self.into_iter()
  }
}
