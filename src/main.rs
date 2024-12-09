#![warn(clippy::pedantic)]
#![deny(clippy::panic)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(osos::runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use osos::{
    memory::{allocator, paging},
    print, println, serial_println,
    task::{executor::Executor, keyboard, Task},
    vga::{self, init_logger},
};
use x86_64::VirtAddr;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // cannot use log crate here for some reason.
    println!("\n\nPANIC: {info}");
    serial_println!("{info}");
    osos::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    osos::test_panic_handler(info);
}

entry_point!(kernel_main);

const LOGGER: vga::Logger = vga::Logger::new(log::LevelFilter::Trace);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    init_logger(&LOGGER);

    print!("Hello, World!");
    println!("!!!~ ");

    serial_println!("Hello, serial0!");

    // init os stuf
    osos::init();

    #[cfg(test)]
    test_main();

    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { paging::init_offset_table(phys_offset) };
    let mut frame_allocator = unsafe { paging::BootInfoFrameAllocator::new(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed");

    // test heap
    {
        let mut suse = alloc::vec![1, 2, 3];
        print!("{suse:?}, ");

        suse.push(10);

        // sleep some
        for _ in 0..2_000_000 {
            x86_64::instructions::nop();
        }

        print!("now {suse:?}, ");

        suse.pop();
        suse.pop();

        for _ in 0..2_000_000 {
            x86_64::instructions::nop();
        }
        println!("now {suse:?}!");
    }

    log::info!("We are done!");
    log::debug!("We are done!");
    log::warn!("We are done!");
    log::error!("We are done!");
    log::trace!("We are done!");

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}
