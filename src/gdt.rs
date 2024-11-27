use core::ptr::addr_of;

use lazy_static::lazy_static;
use x86_64::{
    instructions::tables,
    registers::segmentation::{Segment, CS},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

pub const DOUBLE_FAULT_IST_INDEX: usize = 0;

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code = gdt.append(Descriptor::kernel_code_segment());
        let tss = gdt.append(Descriptor::tss_segment(&TSS));

        (gdt, Selectors { code, tss })
    };
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            #[allow(unused_unsafe)]
            let stack_start = VirtAddr::from_ptr(unsafe { addr_of!(STACK) });

            // stack end
            stack_start + STACK_SIZE as u64
        };
        tss
    };
}

struct Selectors {
    code: SegmentSelector,
    tss: SegmentSelector,
}

pub fn init() {
    GDT.0.load();

    unsafe {
        CS::set_reg(GDT.1.code);
        tables::load_tss(GDT.1.tss);
    }
}
