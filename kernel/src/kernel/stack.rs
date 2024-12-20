use core::mem::MaybeUninit;

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

pub struct StackInfo {
    pub size: usize,
    pub stack_end: *mut u32,
}

impl StackInfo {
    /// Write a value at a specific offset from the bottom of the stack.
    ///
    /// # Arguments
    /// * `offset` - The offset from the bottom of the stack to write the value.
    /// * `obj` - The value to write.
    ///
    /// # Returns
    /// The pointer to the written value if the write was successful, otherwise `None`.
    pub unsafe fn write_obj_at<T: Sized>(&self, offset: usize, obj: T) -> Option<*mut T> {
        let obj_size = size_of::<T>();

        if offset + obj_size <= self.size {
            let stack_end = self.stack_end as *mut u8;
            let ptr = stack_end.sub(self.size - offset) as *mut T;
            ptr.write(obj);

            Some(ptr)
        } else {
            None
        }
    }
}
