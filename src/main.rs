#![warn(clippy::pedantic)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(osos::runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use osos::{memory, print, println, serial_println};
use x86_64::{
    structures::paging::Page,
    VirtAddr,
};

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
    let mut mapper = unsafe { memory::init_offset_table(phys_offset) };
    let mut allocator = memory::EmptyFrameAllocator;

    let page = Page::containing_address(VirtAddr::zero());
    memory::example_mapping(page, &mut mapper, &mut allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // D0 = pink bg, black fg
    // 50 = ascii P
    unsafe {
        page_ptr.offset(400).write_volatile(0xD050_D050);
    }

    // let addrs = [
    //     // identity mapped vga buffer page
    //     0xb8000,
    //     // some code page
    //     0x0020_1008,
    //     // some stack page
    //     0x0100_0020_1a10,
    //     // virt addr mapped to phys addr 0
    //     boot_info.physical_memory_offset,
    // ];

    // let mapper = unsafe { memory::init_offset_table(phys_offset) };

    // for &addr in &addrs {
    //     let virt = VirtAddr::new(addr);
    //     let phys = mapper.translate_addr(virt);
    //     println!("{virt:?} -> {phys:?}");
    // }

    unsafe {
        println!("{}", *(0xfe0e as *const usize));
    }

    #[cfg(test)]
    test_main();

    println!("End");

    osos::hlt_loop();
}
