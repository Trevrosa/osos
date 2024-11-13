use x86_64::{registers::control::Cr3, structures::paging::PageTable, VirtAddr};

/// Return a mutable reference to the active level 4 table
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    // Cr3: control register 3 = contains the physical address of the highest level page table
    let (level_4_table_frame, _cr3flags) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}
