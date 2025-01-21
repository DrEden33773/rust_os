#![no_std]
#![cfg_attr(test, no_main)]
// #![feature(const_mut_refs)] // stable in 1.83.0-nightly
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(async_closure)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod allocator;
pub mod collections;
pub mod demo;
pub mod exit;
pub mod gdt;
pub mod interrupts;
pub mod io;
pub mod memory;
pub mod prelude;
pub mod serial;
pub mod shell;
pub mod task;
pub mod test_framework;
pub mod utils;
pub mod vga_buffer;

#[cfg(test)]
use bootloader::entry_point;
use bootloader::BootInfo;
use core::panic::PanicInfo;
use exit::{exit_qemu, QemuExitCode};
use memory::BootInfoFrameAllocator;
use test_framework::Testable;
use x86_64::VirtAddr;

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo test`
#[cfg(test)]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
  minimum_init(boot_info);
  test_main();
  hlt_loop();
}

pub fn test_runner(tests: &[&dyn Testable]) {
  serial_println!("\nRunning {} tests\n", tests.len());
  for test in tests {
    test.run();
  }
  serial_println!();
  exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
  serial_println!("[failed]\n");
  serial_println!("Error: {}\n", info);
  exit_qemu(QemuExitCode::Failed);
  hlt_loop()
}

pub fn hlt_loop() -> ! {
  loop {
    x86_64::instructions::hlt()
  }
}

#[cfg(test)]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
  test_panic_handler(info)
}

pub fn minimum_init(boot_info: &'static BootInfo) {
  // gdt(tss) init
  gdt::init();
  // idt init
  interrupts::init_idt();
  // PIC init
  unsafe { interrupts::PICS.lock().initialize() };
  // enable listening on PIC
  x86_64::instructions::interrupts::enable();
  // heap init
  let (mut mapper, mut frame_allocator) = {
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mapper = unsafe { memory::init(phys_mem_offset) };
    let frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    (mapper, frame_allocator)
  };
  allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed!\n");
}
