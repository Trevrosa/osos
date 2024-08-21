#![no_std]
#![no_main]

use core::panic::PanicInfo;
use osos::{
    qemu::{self, ExitCode},
    serial_print, serial_println,
};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    qemu::exit(ExitCode::Failed);
    loop {}
}

fn should_fail() {
    serial_print!("should_panic::should_fail.. ");
    panic!();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    qemu::exit(ExitCode::Success);
    loop {}
}
