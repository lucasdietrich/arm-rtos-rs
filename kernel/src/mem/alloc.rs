// Simple bump allocator for the kernel.

use core::{
    alloc::GlobalAlloc,
    cell::UnsafeCell,
    ptr::null_mut,
    sync::atomic::{AtomicUsize, Ordering},
};

const MAX_SUPPORTED_ALIGN: usize = 8;
const KERNEL_ALLOCATOR_SIZE: usize = 4096;

// Align must match MAX_SUPPORTED_ALIGN
#[repr(C, align(8))]
pub struct BumpAllocator<const SIZE: usize> {
    arena: UnsafeCell<[u8; SIZE]>,
    remaining: AtomicUsize,
}

#[global_allocator]
pub static KERNEL_ALLOCATOR: BumpAllocator<KERNEL_ALLOCATOR_SIZE> = BumpAllocator::new();

impl<const SIZE: usize> BumpAllocator<SIZE> {
    pub const fn new() -> Self {
        BumpAllocator {
            arena: UnsafeCell::new([0x77; SIZE]),
            remaining: AtomicUsize::new(SIZE),
        }
    }
}

unsafe impl<const SIZE: usize> Sync for BumpAllocator<SIZE> {}

unsafe impl<const SIZE: usize> GlobalAlloc for BumpAllocator<SIZE> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        if align > MAX_SUPPORTED_ALIGN {
            return null_mut();
        }

        let mut offset = 0;
        if self
            .remaining
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |mut remaining| {
                if remaining < size {
                    return None;
                }

                let align_mask = !(align - 1);

                remaining -= size;
                remaining &= align_mask;
                offset = remaining;
                Some(remaining)
            })
            .is_err()
        {
            return null_mut();
        }

        self.arena.get().cast::<u8>().add(offset)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        /* Bump allocator actually don't deallocate previously allocated memory */
    }
}
