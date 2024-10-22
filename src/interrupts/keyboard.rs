use lazy_static::lazy_static;
use pc_keyboard::{layouts::Us104Key, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use x86_64::{instructions::port::Port, structures::idt::InterruptStackFrame};

use crate::{
    interrupts::{notify_end_of_interrupt, InterruptIndex},
    print, serial_println,
};

pub extern "x86-interrupt" fn handler(_frame: InterruptStackFrame) {
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<Us104Key, ScancodeSet1>> = {
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                Us104Key,
                HandleControl::Ignore,
            ))
        };
    }

    let mut keyboard = KEYBOARD.lock();
    let scancode: u8 = unsafe { Port::new(0x60).read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{character}"),
                // not char keys
                DecodedKey::RawKey(key) => serial_println!("{key:?}"),
            };
        }
    }

    unsafe {
        notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}
