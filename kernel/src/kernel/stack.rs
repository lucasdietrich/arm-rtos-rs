use core::{
    mem::{self, MaybeUninit},
    ptr,
};

use super::errno::Kerr;

/// A fixed-size stack suitable for use as a task stack.
///
/// The `Stack` struct represents a memory region that can be used as a stack for
/// tasks. Use generic type `Z` to specify the size of the stack in bytes.
///
/// This stack definition ensures the initial stack pointer is always 8 bytes aligned.
///
/// # Examples
///
/// ```rust
/// use your_crate::{Stack, StackInfo};
///
/// // Initialize a thread stack with 32 KB size in the `.noinit` section.
/// #[link_section = ".noinit"]
/// static mut THREAD_STACK2: Stack<32768> = Stack::init();
///
/// // Get stack information.
/// let stack2_info = unsafe { &mut THREAD_STACK2 }.get_info();
///
/// // Use the stack information to initialize a new thread (pseudo-code).
/// let task2 = Thread::init(&stack2_info, mytask_entry, 0xbbbb0000 as *mut c_void);
/// ```
#[repr(C, align(8))]
pub struct Stack<const Z: usize> {
    stack: MaybeUninit<[u8; Z]>,
}

impl<const Z: usize> Stack<Z> {
    pub fn size(&self) -> usize {
        Z
    }

    pub fn stack_end_ptr(&mut self) -> *mut u32 {
        // This guarentees the end stack pointer is 8 bytes aligned
        let align8_size = Z - (Z % 8);
        unsafe { self.stack.as_mut_ptr().byte_add(align8_size) as *mut u32 }
    }

    pub const fn zeroed() -> Stack<Z> {
        Stack {
            stack: MaybeUninit::new([0; Z]),
        }
    }

    pub const fn uninit() -> Stack<Z> {
        Stack {
            stack: MaybeUninit::uninit(),
        }
    }

    pub fn get_info(&mut self) -> StackInfo {
        StackInfo {
            size: Z,
            stack_end: self.stack_end_ptr(),
        }
    }
}

#[derive(Clone)]
pub struct StackInfo {
    // Number of bytes in the stack
    pub size: usize,
    pub stack_end: *mut u32,
}

impl StackInfo {
    // Get the stack start pointer
    pub fn stack_start(&self) -> *mut u32 {
        unsafe { self.stack_end.byte_sub(self.size) }
    }

    // Calculate the offset of the given pointer from the stack start, without bounds checking
    pub unsafe fn ptr_to_offset_unchecked(&self, ptr: *mut u32) -> usize {
        let start = self.stack_start();

        let offset = ptr.offset_from(start);

        offset as usize
    }

    // Calculate the pointer within stack of the given offset, without bounds checking
    pub unsafe fn offset_to_ptr_unchecked(&self, offset: usize) -> *mut u32 {
        self.stack_start().byte_add(offset)
    }

    // // Calculate the offset of the given pointer from the stack start
    // pub fn ptr_to_offset(&self, ptr: *mut u32) -> Result<usize, ()> {
    //     unsafe {
    //         let start = self.stack_start();

    //         let offset = ptr.offset_from(start);
    //         if offset < 0 {
    //             return Err(());
    //         }

    //         let offset = offset as usize;
    //         if offset < self.size {
    //             Ok(offset * size_of::<u32>())
    //         } else {
    //             Err(())
    //         }
    //     }
    // }

    // // Calculate the pointer within stack of the given offset
    // pub fn offset_to_ptr(&self, offset: usize) -> Result<*mut u32, ()> {
    //     if offset >= self.size {
    //         return Err(());
    //     }

    //     Ok(unsafe { self.offset_to_ptr_unchecked(offset) })
    // }
}

// Copy the stack from one location to another without overlap or size checking
pub unsafe fn stack_copy_unchecked(from: &StackInfo, to: &StackInfo) {
    ptr::copy_nonoverlapping(
        from.stack_start() as *mut u8,
        to.stack_start() as *mut u8,
        to.size,
    );
}

pub enum StackError {
    Overlapping,
    NoMemory,
}

// Copy the stack from one location to another with overlap or size checking
pub fn stack_copy(from: &StackInfo, to: &StackInfo) -> Result<(), StackError> {
    // Check if the destination stack is smaller than the source stack
    if to.size < from.size {
        return Err(StackError::NoMemory);
    }

    // Determine which stack is lower and which is upper in memory
    let (lower, upper) = if from.stack_start() < to.stack_start() {
        (from, to)
    } else {
        (to, from)
    };

    // Check for overlap
    if lower.stack_end > upper.stack_start() {
        return Err(StackError::Overlapping);
    }

    // Copy the stack
    unsafe {
        stack_copy_unchecked(from, to);
    }

    Ok(())
}
