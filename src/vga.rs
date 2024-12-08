use core::{
    fmt::{self, Write},
    ops::Deref,
};

use conquer_once::spin::Lazy;
use log::{LevelFilter, Log};
use spin::Mutex;
use volatile::Volatile;

const VGA_BUFFER: *mut Buffer = 0xb8000 as *mut Buffer;

pub static WRITER: Lazy<Mutex<Writer>> = Lazy::new(|| {
    let color_code = ColorCode::new(Color::Green, Color::Black);
    let buffer = unsafe { &mut *VGA_BUFFER };
    let writer = Writer::new(color_code, buffer);

    Mutex::new(writer)
});

pub struct Logger {
    pub verbosity: LevelFilter,
}

impl Logger {
    pub const fn new(verbosity: LevelFilter) -> Self {
        Self { verbosity }
    }
}

impl Log for Logger {
    // no way to flush vga
    fn flush(&self) {}

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            crate::println!(
                "{}->{}: {}",
                record.module_path().unwrap(),
                record.level(),
                record.args().as_str().unwrap()
            )
        }
    }

    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.verbosity
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!(format_args!($($arg)*)));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[repr(u8)]
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorCode(u8);

impl Deref for ColorCode {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ColorCode {
    #[must_use]
    pub fn new(foreground: Color, background: Color) -> Self {
        // first 4 bits is background, last 4 is foreground
        // eg. for ColorCode 10100010, the bg is 1010 (10, light green), while the fg is 0010 (2, green)
        Self((background as u8) << 4 | (foreground as u8))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Char {
    ascii_char: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {
    // volatile so compiler doesn't optimize away writes to `chars`
    chars: [[Volatile<Char>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_pos: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

impl Writer {
    pub fn new(color_code: ColorCode, buffer: &'static mut Buffer) -> Self {
        Self {
            column_pos: 0,
            color_code,
            buffer,
        }
    }
}

impl Writer {
    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            if byte.is_ascii() {
                self.write_byte(byte);
            } else {
                // ■ character in vga
                self.write_byte(0xfe);
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        if byte == b'\n' {
            self.new_line();
        } else {
            if self.column_pos >= BUFFER_WIDTH {
                self.new_line();
            }

            let row = BUFFER_HEIGHT - 1;
            let col = self.column_pos;

            self.buffer.chars[row][col].write(Char {
                ascii_char: byte,
                color_code: self.color_code,
            });
            self.column_pos += 1;
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let char = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(char);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_pos = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = Char {
            ascii_char: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

#[test_case]
fn test_println() {
    println!("test output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("hi");
    }
}

#[test_case]
fn test_println_output() {
    let test = "Hello, World!";
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{test}").expect("writeln failed");

        for (i, c) in test.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();

            assert_eq!(char::from(screen_char.ascii_char), c);
        }
    });
}
