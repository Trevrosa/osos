use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
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

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a [`FrameAllocator`] from a passed [`MemoryMap`]
    ///
    /// # Safety
    ///
    /// Caller must guarantee the passed memory map is valid. (all frames marked as `Usable` are actually unused.)
    pub unsafe fn new(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            next: 0,
        }
    }

    /// Get the usable frames from the mapped memory regions
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let usable = self
            .memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable);

        let frame_addrs = usable
            // address regions
            .map(|r| r.range.start_addr()..r.range.end_addr())
            // step 4 kib (4096 bytes) to find the frame start addrs
            .flat_map(|r| r.step_by(4096));

        frame_addrs.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// Initialize a [`OffsetPageTable`]
///
/// # Safety
///
/// - This method may only be called once to avoid aliasing `&mut` references.
/// - The caller must ensure the complete physical memory is mapped to virtual memory at the `phys_offset` given.
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
