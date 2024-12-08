use conquer_once::spin::Lazy;
use pc_keyboard::{layouts::Us104Key, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use x86_64::{instructions::port::Port, structures::idt::InterruptStackFrame};

use crate::interrupt::{notify_end_of_interrupt, InterruptIndex};

static KEYBOARD: Lazy<Mutex<Keyboard<Us104Key, ScancodeSet1>>> = Lazy::new(|| {
    Mutex::new(Keyboard::new(
        ScancodeSet1::new(),
        Us104Key,
        HandleControl::Ignore,
    ))
});

pub extern "x86-interrupt" fn handler(_frame: InterruptStackFrame) {
    let mut keyboard = KEYBOARD.lock();
    let scancode: u8 = unsafe { Port::new(0x60).read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}
