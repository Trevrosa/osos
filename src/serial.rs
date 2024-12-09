#![allow(clippy::module_name_repetitions)]

use core::fmt::{self, Write};

use conquer_once::spin::Lazy;
use spin::Mutex;
use uart_16550::SerialPort;

pub static SERIAL0: Lazy<Mutex<SerialPort>> = Lazy::new(|| {
    let mut serial_port = unsafe { SerialPort::new(0x3F8) };
    serial_port.init();
    Mutex::new(serial_port)
});

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::serial::private_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!(format_args!($($arg)*)));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn private_print(args: fmt::Arguments) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        SERIAL0.lock().write_fmt(args).unwrap();
    });
}
