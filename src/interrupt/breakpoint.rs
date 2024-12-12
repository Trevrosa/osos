use log::error;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn handler(frame: InterruptStackFrame) {
    error!("breakpoint exception received: {frame:#?}");
}
