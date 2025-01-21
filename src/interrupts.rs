use crate::{gdt, hlt_loop, print, println, vga_buffer::WRITER};
use lazy_static::lazy_static;
use pc_keyboard::KeyCode;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Intel 8259 Compatible PIC
pub static PICS: Mutex<ChainedPics> =
  Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

/// hook of `breakpoint`
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
  println!("EXCEPTION: BREAKPOINT\n{:#?}\n", stack_frame);
}

/// hook of `double_fault`
extern "x86-interrupt" fn double_fault_handler(
  stack_frame: InterruptStackFrame,
  _error_code: u64,
) -> ! {
  panic!("EXCEPTION: DOUBLE FAULT\n{:#?}\n", stack_frame);
}

/// hook of `timer_interrupt`
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
  // print!(".");
  // handle `EOI`
  unsafe {
    PICS
      .lock()
      .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
  }
}

/// hook of `keyboard_interrupt`
#[deprecated = "Should use `async` handler"]
#[allow(dead_code)]
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
  use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
  use x86_64::instructions::port::Port;

  // KEYBOARD Pool
  lazy_static! {
      static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
          Mutex::new(Keyboard::new(
              ScancodeSet1::new(), // Set-1
              layouts::Us104Key, // US-104-Key keyboard
              HandleControl::Ignore // Ignore mapping to Unicode
          ));
  }

  // keyboard singleton
  let mut keyboard = KEYBOARD.lock();

  // port <~ 0x60 (IO)
  let mut port = Port::new(0x60);

  // scancode
  let scancode: u8 = unsafe { port.read() };

  if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
    if let Some(key) = keyboard.process_keyevent(key_event) {
      match key {
        // input := <backspace>
        DecodedKey::Unicode(character) if character as u8 == b'\x08' => {
          x86_64::instructions::interrupts::without_interrupts(|| {
            WRITER.lock().enforce_backspace();
          })
        }
        // input := unicode_char
        DecodedKey::Unicode(character) => print!("{}", character),
        // input <~ human-readable event (e.g. press `CapsLock` or 'LCtrl')
        DecodedKey::RawKey(key) => match key {
          KeyCode::Backspace => x86_64::instructions::interrupts::without_interrupts(|| {
            WRITER.lock().enforce_backspace();
          }),
          KeyCode::LControl | KeyCode::RControl => print!("^"),
          _ => {}
        },
      }
    }
  }

  // handle `EOI`
  unsafe {
    PICS
      .lock()
      .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
  }
}

/// hook of `keyboard_interrupt`, with support of concurrency
extern "x86-interrupt" fn async_keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
  use x86_64::instructions::port::Port;

  let mut port = Port::new(0x60);
  let scancode: u8 = unsafe { port.read() };

  crate::task::keyboard::add_scancode(scancode);

  // handle `EOI`
  unsafe {
    PICS
      .lock()
      .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
  }
}

/// hook of `page_fault`
extern "x86-interrupt" fn page_fault_handler(
  stack_frame: InterruptStackFrame,
  error_code: PageFaultErrorCode,
) {
  use x86_64::registers::control::Cr2;

  println!("\nEXCEPTION: PAGE FAULT");
  println!("Accessed Address: {:?}", Cr2::read());
  println!("Error Code: {:?}", error_code);
  println!("{:#?}\n", stack_frame);
  hlt_loop();
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
  Timer = PIC_1_OFFSET, // offset = 0
  Keyboard,             // offset = +1
}

impl InterruptIndex {
  fn as_u8(self) -> u8 {
    self as u8
  }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        // init
        let mut idt = InterruptDescriptorTable::new();
        // breakpoint
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // double_fault (with a pre-defined reserved stack)
        unsafe { idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX) };
        // timer_interruption
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        // keyboard_interruption
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(async_keyboard_interrupt_handler);
        // page_fault
        idt.page_fault.set_handler_fn(page_fault_handler);
        // ref bind
        idt
    };
}

pub fn init_idt() {
  IDT.load();
}

#[test_case]
fn test_breakpoint_exception() {
  // invoke a breakpoint exception
  x86_64::instructions::interrupts::int3();
}
