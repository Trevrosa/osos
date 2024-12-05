#![warn(clippy::pedantic)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(osos::runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{format, string::String};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use osos::{
    memory::{allocator, paging},
    print, println, serial_println,
    task::{executor::SimpleExecutor, Task},
};
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

    #[cfg(test)]
    test_main();

    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { paging::init_offset_table(phys_offset) };
    let mut frame_allocator = unsafe { paging::BootInfoFrameAllocator::new(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed");

    // test heap
    {
        let mut suse = alloc::vec![1, 2, 3];
        print!(" {suse:?}, ");
        
        suse.push(10);
        
        // sleep some
        for _ in 0..2000000 {
            x86_64::instructions::nop();
        }
        
        print!("now {suse:?}, ");
        
        suse.pop();
        suse.pop();
        
        for _ in 0..2000000 {
            x86_64::instructions::nop();
        }
        println!("now {suse:?}!");
    }

    // test async
    {
        async fn text() -> String {
            unsafe { format!("{}", *(0xfe0e as *const usize)) }
        }
    
        async fn test_async() {
            let num = text().await;
            println!("async {num}");
        }
    
        let mut executor = SimpleExecutor::new();
        executor.spawn(Task::new(test_async()));
        executor.run();
    }

    // test example memory mapping
    {
        let page = x86_64::structures::paging::Page::containing_address(VirtAddr::zero());
        let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    
        paging::example_mapping(page, &mut mapper, &mut frame_allocator);
    
        for _ in 0..5000000 {
            x86_64::instructions::nop();
        }
    
        for i in 1..=500 {
            unsafe {
                // each 4 hex digit is a vga char
                // first 2 hex digits (from left) = fg and bg bytes (see vga::Char)
                // last 2 hex digits (from left) = ascii code point
                page_ptr.offset(i).write_volatile(0xD354_D050_D041);
            }
    
            for _ in 0..10000 {
                x86_64::instructions::nop();
            }
        }    
    }

    println!("We are done!");
    osos::hlt_loop();
}
