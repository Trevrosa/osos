use x86_64::structures::idt::InterruptStackFrame;

#[allow(clippy::panic, reason = "double faults are unrecoverable")]
pub extern "x86-interrupt" fn handler(frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!("NOOOOO DOUBLE FAULT {error_code} (panicking): {frame:#?}");
}
