use x86_64::structures::idt::InterruptStackFrame;

use crate::println;

pub extern "x86-interrupt" fn handler(frame: InterruptStackFrame) {
    println!("EXCEPTION!! BREAKPOINT: {frame:#?}");
}
