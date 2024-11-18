use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        page_table::FrameError::{FrameNotPresent, HugeFrame},
        PageTable,
    },
    PhysAddr, VirtAddr,
};

/// Return a mutable reference to the active level 4 table
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    // Cr3: control register 3 = contains the physical address of the highest level page table
    let (level_4_table_frame, _cr3flags) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

/// Translate a given virtual address to a mapped physical address
/// or `None` if the address is not mapped.
/// 
/// # Safety
/// 
/// The caller must guarantee that the complete physical memory
/// is mapped to virtual memory at the passed `phys_offset`.
pub unsafe fn translate_addr(addr: VirtAddr, phys_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, phys_offset)
}

/// Inner function for `translate_addr`.
fn translate_addr_inner(addr: VirtAddr, phys_offset: VirtAddr) -> Option<PhysAddr> {
    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let (mut frame, _) = Cr3::read();

    for &index in &table_indexes {
        let virt = phys_offset + frame.start_address().as_u64();

        let table = virt.as_ptr();
        let table: &PageTable = unsafe { &*table };

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameNotPresent) => return None,
            Err(HugeFrame) => panic!("hugepages not suported.."),
        };
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}