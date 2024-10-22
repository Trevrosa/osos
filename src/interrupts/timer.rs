use x86_64::structures::idt::InterruptStackFrame;

use crate::{
    interrupts::{notify_end_of_interrupt, InterruptIndex},
    print,
};

pub extern "x86-interrupt" fn handler(_frame: InterruptStackFrame) {
    print!(".");

    unsafe {
        notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}
