mod breakpoint;
mod double_fault;
mod keyboard;
mod timer;

use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::InterruptDescriptorTable;

use crate::gdt;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    // timer + 1
    Keyboard,
}

pub fn init_idt() {
    IDT.load();
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint::handler);

        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault::handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX as u16);
        }

        idt[InterruptIndex::Timer as usize].set_handler_fn(timer::handler);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard::handler);

        idt
    };
}

#[inline]
unsafe fn notify_end_of_interrupt(interrupt_id: u8) {
    PICS.lock().notify_end_of_interrupt(interrupt_id);
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
