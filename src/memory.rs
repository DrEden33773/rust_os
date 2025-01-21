use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
  structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
  },
  PhysAddr, VirtAddr,
};

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
  fn allocate_frame(&mut self) -> Option<PhysFrame> {
    None
  }
}

/// ## BootInfoFrameAllocator
///
/// A `FrameAllocator` which gets available frames from bootloader's memory map
pub struct BootInfoFrameAllocator {
  memory_map: &'static MemoryMap,
  next: usize,
}

impl BootInfoFrameAllocator {
  /// memory_map => FrameAllocator
  ///
  /// # Safety
  ///
  /// Unsafe (reason: caller must ensure `memory_map` is available)
  ///
  /// (in another word, `available`-marked frame should be unused)
  pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
    BootInfoFrameAllocator {
      memory_map,
      next: 0,
    }
  }
}

impl BootInfoFrameAllocator {
  /// ## usable_frames
  ///
  /// Return available iterator of `PhysFrame` in `memory_map`
  fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
    // usable_regions <~ memory_map (get)
    let regions = self.memory_map.iter();
    let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
    // usable_regions => addr_ranges (convert)
    let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
    // addr_ranges ==(flatten)=> frame_address (convert)
    // `4096` := sizeof(page)
    let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
    // `PhysFrame`
    frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
  }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
  fn allocate_frame(&mut self) -> Option<PhysFrame> {
    // get available frame
    let frame = self.usable_frames().nth(self.next);
    // update the mark
    self.next += 1;
    // return
    frame
  }
}

/// create an example mapping to `0xb8000` => VGA_BUFFER
pub fn create_example_mapping(
  page: Page,
  mapper: &mut OffsetPageTable,
  frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
  use x86_64::structures::paging::PageTableFlags as Flags;

  let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
  let flags = Flags::PRESENT | Flags::WRITABLE;

  let map_to_result = unsafe {
    // unsafe (works well `IFF` call once)
    mapper.map_to(page, frame, flags, frame_allocator)
  };
  map_to_result.expect("map_to failed!\n").flush();
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
  use x86_64::registers::control::Cr3;

  let (level_4_table_frame, _) = Cr3::read();

  let phys = level_4_table_frame.start_address();
  let virt = physical_memory_offset + phys.as_u64();
  let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

  &mut *page_table_ptr
}

/// # Safety
///
/// Unsafe (could only called once)
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
  let level_4_table = active_level_4_table(physical_memory_offset);
  OffsetPageTable::new(level_4_table, physical_memory_offset)
}

#[cfg(feature = "usr_def_addr_translate")]
fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
  use x86_64::{registers::control::Cr3, structures::paging::page_table::FrameError};

  // CR3 ->> 4 level active table frames
  let (level_4_table_frame, _) = Cr3::read();

  let table_indexes = [
    addr.p4_index(),
    addr.p3_index(),
    addr.p2_index(),
    addr.p1_index(),
  ];
  let mut frame = level_4_table_frame;

  // iterate index of each table
  for &index in &table_indexes {
    // get current table (from the last frame)
    let virt = physical_memory_offset + frame.start_address().as_u64();
    let table_ptr: *const PageTable = virt.as_ptr();
    let table = unsafe { &*table_ptr };

    // read `table` entry -> update `frame`
    let entry = &table[index];
    frame = match entry.frame() {
      Ok(frame) => frame,
      Err(FrameError::FrameNotPresent) => return None,
      Err(FrameError::HugeFrame) => panic!("huge pages not supported!\n"),
    };
  }

  // virtual_addr + offset = physical_addr
  Some(frame.start_address() + u64::from(addr.page_offset()))
}

#[cfg(feature = "usr_def_addr_translate")]
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
  translate_addr_inner(addr, physical_memory_offset)
}
