#![warn(clippy::pedantic)]
#![deny(clippy::panic)]
#![no_std]
#![feature(abi_x86_interrupt)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod qemu;
pub mod serial;
pub mod vga;

pub mod gdt;
pub mod interrupt;

pub mod memory;

pub mod task;

use core::{any, panic::PanicInfo};

pub type Board = Vec<Vec<Option<bool>>>;
use alloc::vec::Vec;

pub static BOARD: spin::Mutex<Board> = spin::Mutex::new(Vec::new());

pub static STATE: spin::Mutex<bool> = spin::Mutex::new(true);

pub fn print_board() {
    println!();

    let board = BOARD.lock();
    for i in 0..=2 {
        let row: &[Option<bool>] = &board[i];
        for j in 0..=2 {
            let col: &Option<bool> = &row[j];
            let col = match col {
                Some(c) => {
                    if *c {
                        "X"
                    } else {
                        "O"
                    }
                }
                None => "_",
            };
            print!("{col}");
        }
        println!();
    }
}
pub fn clear_board() {
    let mut board = BOARD.lock();
    board.clear();
    board.push([None, None, None].to_vec());
    board.push([None, None, None].to_vec());
    board.push([None, None, None].to_vec());
}
use log::info;
use x86_64::instructions;

/// initialize
/// - gdt
/// - idt
/// - PICs
/// - interrupts
pub fn init() {
    info!("first init");
    gdt::init();
    interrupt::init_idt();
    unsafe { interrupt::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

#[inline]
pub fn hlt_loop() -> ! {
    loop {
        instructions::hlt();
    }
}

pub trait Testable {
    fn run(&self);
}

impl<T: Fn()> Testable for T {
    fn run(&self) {
        serial_print!("{}.. ", any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    qemu::exit(qemu::ExitCode::Success);
}

#[cfg(test)]
bootloader::entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    interrupt::init_idt();
    test_main();
    hlt_loop();
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("{info}\n");

    qemu::exit(qemu::ExitCode::Failed);

    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
