#![warn(clippy::pedantic)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(osos::runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use osos::{print, println, serial_println};

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    osos::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    osos::test_panic_handler(info);
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print!("Hello, World!");
    print!("!!!~ ");

    serial_println!("Hello, serial0!");

    // init stuf
    osos::init();

    unsafe {
        println!("{}", *(0xfe0e as *const usize));
    }

    // // breakpoint int
    // x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    println!("dunne");

    osos::hlt_loop();
}
