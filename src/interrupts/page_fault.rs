use x86_64::{
    registers::control::Cr2,
    structures::idt::{InterruptStackFrame, PageFaultErrorCode},
};

use crate::{hlt_loop, println};

pub extern "x86-interrupt" fn handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    println!("EXCEPTION: PAGE FAULT");
    // CR2: control register 2 - contains address which triggered the page fault.
    println!("acessed address: {:?}", Cr2::read());
    println!("error code: {error_code:?}");
    println!("{stack_frame:#?}");
    
    hlt_loop();
}
