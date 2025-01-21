use crate::{serial_print, serial_println};

pub trait Testable {
  fn run(&self);
}

impl<T: Fn()> Testable for T {
  fn run(&self) {
    serial_print!("{} ... ", core::any::type_name::<T>());
    self();
    // green `[ok]`
    serial_print!("\x1b[32m");
    serial_print!("[ok]");
    serial_println!("\x1b[0m");
    // red `[failed]`
    // serial_print!("\x1b[31m");
    // serial_print!("[failed]");
    // serial_println!("\x1b[0m");
  }
}
