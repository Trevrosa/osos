use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame,
        Size4KiB,
    },
    PhysAddr, VirtAddr,
};

/// Return a mutable reference to the active level 4 table
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    // Cr3: control register 3 = contains the physical address of the highest level page table
    let (level_4_table_frame, _cr3flags) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

/// Frame Allocator that only returns `None`
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

/// Initialize a [`OffsetPageTable`]
///
/// # Safety
///
/// - This method may only be called once to avoid aliasing `&mut` references.
/// - The caller must ensure the complete physical memory is mapped to virtual memory at the `physical_memory_offset` given.
pub unsafe fn init_offset_table(phys_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(phys_offset);
    OffsetPageTable::new(level_4_table, phys_offset)
}

pub fn example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let frame: PhysFrame = PhysFrame::containing_address(PhysAddr::new(0xb8000)); // vga buffer
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let map_to_result = unsafe {
        // not safe
        mapper.map_to(page, frame, flags, frame_allocator)
    };

    map_to_result.expect("map_to failed").flush();
}
