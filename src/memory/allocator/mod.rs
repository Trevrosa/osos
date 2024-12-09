pub mod fixed_size_block;

use core::ops::Deref;

use log::trace;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 kib

#[global_allocator]
static ALLOCATOR: Locked<fixed_size_block::Allocator> =
    Locked::new(fixed_size_block::Allocator::new());

/// A wrapper around `spin::Mutex` to permit trait implementations.
pub struct Locked<A>(spin::Mutex<A>);

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked(spin::Mutex::new(inner))
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.0.lock()
    }
}

impl<A> Deref for Locked<A> {
    type Target = spin::Mutex<A>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Initialize the heap
///
/// # Errors
///
/// Will error if the memory mapping fails. See [`MapToError`]
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    trace!("initialising heap");
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;

        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);

        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}
