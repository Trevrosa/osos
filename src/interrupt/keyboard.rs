use x86_64::{instructions::port::Port, structures::idt::InterruptStackFrame};

use crate::interrupt::{notify_end_of_interrupt, InterruptIndex};

pub extern "x86-interrupt" fn handler(_frame: InterruptStackFrame) {
    let scancode: u8 = unsafe { Port::new(0x60).read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}
