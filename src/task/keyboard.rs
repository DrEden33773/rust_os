use crate::{eprintln, print, vga_buffer::WRITER};
use conquer_once::spin::OnceCell;
use core::{
  pin::Pin,
  task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{
  stream::{Stream, StreamExt},
  task::AtomicWaker,
};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1};

lazy_static! {
  static ref SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
}
lazy_static! {
  static ref WAKER: AtomicWaker = AtomicWaker::new();
}

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub fn add_scancode(scancode: u8) {
  if let Ok(queue) = SCANCODE_QUEUE.try_get() {
    if queue.push(scancode).is_err() {
      eprintln!("WARNING: `scancode queue` full, dropping keyboard input");
    } else {
      WAKER.wake(); // wake
    }
  } else {
    // eprintln!("WARNING: `scancode queue` uninitialized");
  }
}

pub struct ScancodeStream {
  _private: (),
}

impl ScancodeStream {
  pub fn new() -> Self {
    SCANCODE_QUEUE
      .try_init_once(|| ArrayQueue::new(100))
      .expect("`ScancodeStream::new` should only be called once!\n");
    ScancodeStream { _private: () }
  }
}

impl Default for ScancodeStream {
  fn default() -> Self {
    Self::new()
  }
}

impl Stream for ScancodeStream {
  type Item = u8;

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
    let queue = SCANCODE_QUEUE
      .try_get()
      .expect("scancode_queue not initialized!\n");

    // fast path
    if let Some(scancode) = queue.pop() {
      return Poll::Ready(Some(scancode));
    }

    WAKER.register(cx.waker());
    match queue.pop() {
      Some(scancode) => {
        WAKER.take();
        Poll::Ready(Some(scancode))
      }
      None => Poll::Pending,
    }
  }
}

pub async fn print_keypresses() {
  let mut scancodes = ScancodeStream::new();
  let mut keyboard = Keyboard::new(
    ScancodeSet1::new(),
    layouts::Us104Key,
    HandleControl::Ignore,
  );

  while let Some(scancode) = scancodes.next().await {
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
  }
}
