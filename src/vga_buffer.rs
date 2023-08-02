use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

/// An `enum` type to give a `Color <-> u8` representation map
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Black,
            1 => Self::Blue,
            2 => Self::Green,
            3 => Self::Cyan,
            4 => Self::Red,
            5 => Self::Magenta,
            6 => Self::Brown,
            7 => Self::LightGray,
            8 => Self::DarkGray,
            9 => Self::LightBlue,
            10 => Self::LightGreen,
            11 => Self::LightCyan,
            12 => Self::LightRed,
            13 => Self::Pink,
            14 => Self::Yellow,
            15 => Self::White,
            _ => Self::Black,
        }
    }
}

/// A combination of `foreground` and `background` color, which satisfies:
///
/// ```rust
/// self.0 = (background_color as u8) << 4 | (foreground_color as u8)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

#[allow(dead_code)]
impl ColorCode {
    fn new(foreground: Color, background: Color) -> Self {
        Self(((background as u8) << 4) | (foreground as u8))
    }

    fn new_raw(foreground: u8, background: u8) -> Self {
        Self((background << 4) | foreground)
    }

    fn decrypt(&self) -> (u8, u8) {
        (self.0 & 0x0F, (self.0 & 0xF0) >> 4)
    }

    fn get_foreground(&self) -> u8 {
        self.decrypt().0
    }

    fn get_background(&self) -> u8 {
        self.decrypt().1
    }

    fn set_foreground(&mut self, foreground: Color) {
        self.0 = (self.get_background() << 4) | (foreground as u8);
    }

    fn set_background(&mut self, background: Color) {
        self.0 = ((background as u8) << 4) | self.get_foreground();
    }
}

impl Default for ColorCode {
    /// Default color combination (foreground := white, background := black)
    fn default() -> Self {
        Self::new(Color::White, Color::Black)
    }
}

/// Character displayed on screen, with `ascii_char` and `color_code` info
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// VGA Buffer
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl Buffer {
    /// Initialize VGA Buffer (return a &'static mut Self) only once
    unsafe fn static_init() -> &'static mut Self {
        &mut *(0xb8000 as *mut Buffer)
    }
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { Buffer::static_init() },
    });
}

impl Writer {
    /// Write a byte on the screen (in one line)
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// Add a new line on the screen
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Clear the lowest row (mostly used after called `vga_buffer::Writer::new_line()`)
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl Writer {
    /// Write all bytes in a string on the screen
    /// (via calling `vga_buffer::Writer::write_byte()`)
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // ASCII or '\n' => write it
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Illegal => write `■`
                _ => self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.write_byte(c as u8);
        Ok(())
    }
}

impl Writer {
    fn write_fmt(mut self: &mut Self, args: fmt::Arguments<'_>) -> fmt::Result {
        fmt::write(&mut self, args)
    }
}

pub fn safe_print(args: fmt::Arguments) {
    use x86_64::instructions::interrupts;

    // access WRITER without being interrupted by signals
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

pub fn safe_eprint(args: fmt::Arguments) {
    use x86_64::instructions::interrupts;

    // access WRITER without being interrupted by signals
    interrupts::without_interrupts(|| {
        // WRITER.lock().write_fmt(args).unwrap();
        let mut writer = WRITER.lock();
        let foreground_before = writer.color_code.get_foreground();
        writer.color_code.set_foreground(Color::Yellow);
        writer.write_fmt(args).unwrap();
        writer.color_code.set_foreground(foreground_before.into());
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::safe_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::vga_buffer::safe_eprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprintln {
    () => ($crate::eprint!("\n"));
    ($($arg:tt)*) => ($crate::eprint!("{}\n", format_args!($($arg)*)));
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..50 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use x86_64::instructions::interrupts;

    let s = "A testing string which is in one line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        /*
          `\n` => make sure current line starts with `` instead of `.`
          caused by the timer
        */
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
