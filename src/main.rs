#![warn(clippy::pedantic)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(osos::runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use osos::{memory::translate_addr, print, println, serial_println};
use x86_64::VirtAddr;

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

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    print!("Hello, World!");
    print!("!!!~ ");

    serial_println!("Hello, serial0!");

    // init stuf
    osos::init();

    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let addrs = [
        // identity mapped vga buffer page
        0xb8000,
        // some code page
        0x0020_1008,
        // some stack page
        0x0100_0020_1a10,
        // virt addr mapped to phys addr 0
        boot_info.physical_memory_offset,
    ];

    for &addr in &addrs {
        let virt = VirtAddr::new(addr);
        let phys = unsafe { translate_addr(virt, phys_offset) };
        println!("{virt:?} -> {phys:?}");
    }

    // traverse l4 table
    // let l4_table = unsafe {
    //     let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    //     memory::active_level_4_table(phys_mem_offset)
    // };
    // for (i, entry) in l4_table.iter().enumerate() {
    //     if !entry.is_unused() {
    //         println!("L4 Entry {i}: {entry:?}");
    //     }
    // }

    // access memory outside our kernel
    // let a = 0xdeadbeef as *mut u8;
    // unsafe {
    //     *a = 42;
    // }

    unsafe {
        println!("{}", *(0xfe0e as *const usize));
    }

    // // breakpoint int
    // x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    println!("done");

    osos::hlt_loop();
}
