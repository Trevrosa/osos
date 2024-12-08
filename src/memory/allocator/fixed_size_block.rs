use core::{
    alloc::{GlobalAlloc, Layout},
    mem,
    ptr::{self, NonNull},
};

use log::error;

use super::Locked;

struct ListNode {
    #[allow(dead_code)]
    next: Option<&'static mut ListNode>,
}

/// Block sizes to use
///
/// The sizes must be powers of 2 since we use them as block alignment too (alignment has to be power of 2)
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

pub struct Allocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

/// Get the index of the block size needed for a given `layout`
fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

impl Allocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        Self {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    /// Initialize the fallback allocator with the given heap bounds.
    ///
    /// # Safety
    ///
    /// Caller must guarantee that:
    /// - The given heap bounds are valid.
    /// - The heap is unused.
    /// - This function is called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator
            .init(heap_start as *mut u8, heap_size);
    }

    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        if let Ok(ptr) = self.fallback_allocator.allocate_first_fit(layout) {
            ptr.as_ptr()
        } else {
            error!(
                "fallback allocator failed to allocate {} bytes",
                layout.size()
            );
            ptr::null_mut()
        }
    }
}

unsafe impl GlobalAlloc for Locked<Allocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();

        if let Some(idx) = list_index(&layout) {
            if let Some(node) = allocator.list_heads[idx].take() {
                node as *mut ListNode as *mut u8
            } else {
                let block_size = BLOCK_SIZES[idx];
                let block_align = block_size;

                let layout = Layout::from_size_align(block_size, block_align)
                    .expect("ERROR: failed to create layout when allocating");
                allocator.fallback_alloc(layout)
            }
        } else {
            allocator.fallback_alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();

        if let Some(index) = list_index(&layout) {
            let new_node = ListNode {
                next: allocator.list_heads[index].take(),
            };

            // verify that block has size and alignment required for storing node
            assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
            assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);

            let new_node_ptr = ptr as *mut ListNode;
            new_node_ptr.write(new_node);

            allocator.list_heads[index] = Some(&mut *new_node_ptr);
        } else {
            let ptr = NonNull::new(ptr).unwrap();

            allocator.fallback_allocator.deallocate(ptr, layout);
        }
    }
}
