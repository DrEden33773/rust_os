#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use ember_os::{
  exit::{exit_qemu, QemuExitCode},
  serial_print, serial_println,
};

entry_point!(main);

#[no_mangle]
fn main(_boot_info: &'static BootInfo) -> ! {
  should_fail();

  // red
  serial_print!("\x1b[31m");
  serial_print!("[test did not panic]");
  serial_println!("\x1b[0m");

  exit_qemu(QemuExitCode::Failed);
  ember_os::hlt_loop()
}

fn should_fail() {
  serial_print!("\nshould_panic::should_fail ... ");
  assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  // green
  serial_print!("\x1b[32m");
  serial_print!("[ok]");
  serial_print!("\x1b[0m");
  serial_println!("\n");

  exit_qemu(QemuExitCode::Success);
  ember_os::hlt_loop()
}
