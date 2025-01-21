#![allow(deprecated)]

use super::{align_up, Locked};
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{self};

/// Stack Allocator
#[deprecated = "Fast! But often causes `OOM` (out of memory)"]
pub struct BumpAllocator {
  /// bottom
  heap_start: usize,
  /// surface
  heap_end: usize,
  /// top
  next: usize,
  /// number of allocated memory chunk
  allocations: usize,
}

impl BumpAllocator {
  /// Creates a new `empty` bump allocator
  pub const fn new() -> Self {
    BumpAllocator {
      heap_start: 0,
      heap_end: 0,
      next: 0,
      allocations: 0,
    }
  }

  /// Initializes the bump allocator with the given heap bounds.
  ///
  /// # Safety
  ///
  /// This method is `unsafe` because the caller must ensure that the given
  /// memory range is `unused`. Also, this method must be called `only once`.
  pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
    self.heap_start = heap_start;
    self.heap_end = heap_start + heap_size;
    self.next = heap_start;
  }
}

impl Default for BumpAllocator {
  fn default() -> Self {
    Self::new()
  }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
  /// Allocate on the global bump allocator
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    let mut bump = self.lock();

    let alloc_start = align_up(bump.next, layout.align());
    let alloc_end = match alloc_start.checked_add(layout.size()) {
      Some(end) => end,
      None => return ptr::null_mut(),
    };

    if alloc_end > bump.heap_end {
      // handle OOM (out of memory)
      ptr::null_mut()
    } else {
      bump.next = alloc_end;
      bump.allocations += 1;
      alloc_start as *mut u8
    }
  }

  /// Deallocate the global bump allocator
  ///
  /// This function only decrease the `allocation_counter`,
  /// which trigger `free_all` iff `allocation_counter = 0`
  unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
    let mut bump = self.lock();

    bump.allocations -= 1;

    if bump.allocations == 0 {
      bump.next = bump.heap_start;
    }
  }
}
