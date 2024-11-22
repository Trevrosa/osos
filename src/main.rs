#![warn(clippy::pedantic)]
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
};
use x86_64::{structures::paging::Page, VirtAddr};

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
    let mut mapper = unsafe { paging::init_offset_table(phys_offset) };
    let mut frame_allocator = unsafe { paging::BootInfoFrameAllocator::new(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed");

    // test heap
    let mut suse = alloc::vec![1, 2, 3];
    print!(" {suse:?}, ");
    suse.push(10);
    println!("now {suse:?}");

    // test example mapping
    let page = Page::containing_address(VirtAddr::zero());
    paging::example_mapping(page, &mut mapper, &mut frame_allocator);
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe {
        // D0 = pink bg, black fg
        // 50 = ascii P
        page_ptr.offset(400).write_volatile(0xD050_D050);
    }

    unsafe {
        println!("{}", *(0xfe0e as *const usize));
    }

    #[cfg(test)]
    test_main();

    println!("End");

    osos::hlt_loop();
}
