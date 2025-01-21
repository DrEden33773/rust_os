#![allow(dead_code)]

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use x86_64::{
  structures::paging::{
    mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
  },
  VirtAddr,
};

pub mod bump;
pub mod fixed_size_block;
pub mod linked_list;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 512 * 1024; // 512 KiB
pub const HEAP_START_PTR: *mut u8 = HEAP_START as *mut u8;

/// `zero-sized` type
pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
  unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
    null_mut()
  }
  unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
    panic!("dealloc should be never called!\n")
  }
}

/// A wrapper around spin::Mutex to permit trait implementations.
pub struct Locked<T> {
  inner: spin::Mutex<T>,
}

impl<T> Locked<T> {
  /// Create a lock
  pub const fn new(inner: T) -> Self {
    Locked {
      inner: spin::Mutex::new(inner),
    }
  }

  /// Get the lock
  pub fn lock(&self) -> spin::MutexGuard<T> {
    self.inner.lock()
  }
}

/// Align the given address `addr` upwards to alignment `align`.
#[deprecated]
#[allow(dead_code)]
#[no_mangle]
fn slow_align_up(addr: usize, align: usize) -> usize {
  let remainder = addr % align;
  if remainder == 0 {
    addr // addr already aligned
  } else {
    addr - remainder + align
  }
}

/// Align the given address `addr` upwards to alignment `align`.
///
/// Requires that `align` is a power of two.
#[no_mangle]
fn align_up(addr: usize, align: usize) -> usize {
  let offset = (addr as *const u8).align_offset(align);
  addr + offset
}

cfg_if::cfg_if! {
  if #[cfg(feature = "use_FixedSizeBlockAllocator")] {

    use fixed_size_block::FixedSizeBlockAllocator;

    #[global_allocator]
    static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());

  } else if #[cfg(feature = "use_LockedHeapAllocator")] {

    use linked_list_allocator::LockedHeap;

    #[global_allocator]
    static ALLOCATOR: LockedHeap = LockedHeap::empty();

  } else if #[cfg(feature = "use_LinkedListAllocator")] {

    use linked_list::LinkedListAllocator;

    #[global_allocator]
    static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

  } else if #[cfg(feature = "use_BumpAllocator")] {

    use bump::BumpAllocator;

    #[global_allocator]
    static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

  } else {

    use fixed_size_block::FixedSizeBlockAllocator;

    #[global_allocator]
    static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());

  }
}

pub fn init_heap(
  mapper: &mut impl Mapper<Size4KiB>,
  frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
  // get page_range
  let page_range = {
    let heap_start = VirtAddr::new(HEAP_START as u64);
    let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
    let heap_start_page = Page::containing_address(heap_start);
    let heap_end_page = Page::containing_address(heap_end);
    Page::range_inclusive(heap_start_page, heap_end_page)
  };

  // map all heap pages to physical frames
  for page in page_range {
    let frame = frame_allocator
      .allocate_frame()
      .ok_or(MapToError::FrameAllocationFailed)?;
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
  }

  // init `ALLOCATOR`
  unsafe {
    ALLOCATOR.lock().init(HEAP_START_PTR, HEAP_SIZE);
  }

  Ok(())
}
