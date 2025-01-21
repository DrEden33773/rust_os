#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use ember_os::{
  exit::{exit_qemu, QemuExitCode},
  serial_print, serial_println,
};
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  ember_os::test_panic_handler(info)
}

lazy_static! {
  static ref TEST_IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    unsafe {
      idt
        .double_fault
        .set_handler_fn(test_double_fault_handler)
        .set_stack_index(ember_os::gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt
  };
}

extern "x86-interrupt" fn test_double_fault_handler(
  _stack_frame: InterruptStackFrame,
  _error_code: u64,
) -> ! {
  // green
  serial_print!("\x1b[32m");
  serial_print!("[ok]");
  serial_print!("\x1b[0m");
  serial_println!("\n");

  exit_qemu(QemuExitCode::Success);
  ember_os::hlt_loop()
}

pub fn init_test_idt() {
  TEST_IDT.load();
}

entry_point!(main);

#[no_mangle]
fn main(_boot_info: &'static BootInfo) -> ! {
  serial_print!("\nstack_overflow::stack_overflow ... ");

  ember_os::gdt::init();
  init_test_idt();

  // trigger a stack overflow
  stack_overflow();

  // red
  serial_print!("\x1b[31m");
  serial_print!("[test did not panic]");
  serial_println!("\x1b[0m");

  panic!("execution continued after stack overflow!\n");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
  // recursion bomb
  stack_overflow();
  // prevent tail recursion optimizations
  volatile::Volatile::new(0).read();
}
