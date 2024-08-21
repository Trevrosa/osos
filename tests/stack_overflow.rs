#![no_std]
#![no_main]

use core::panic::PanicInfo;
use osos::{
    qemu::{self, ExitCode},
    serial_print, serial_println,
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    qemu::exit(ExitCode::Success);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow.. ");

    osos::init();

    // trigger a stack overflow
    stack_overflow();

    // stack overflow should cause a panic in its handler
    // so if it didnt something went wrong
    serial_println!("[failed]");
    qemu::exit(ExitCode::Failed);

    loop {}
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
    volatile::Volatile::new(0).read(); // prevent tail recursion optimizations
}
