/// A fixed-size stack suitable
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
    stack: [u8; Z],
}

impl<const Z: usize> Stack<Z> {
    pub fn size(&self) -> usize {
        Z
    }

    pub fn stack_end_ptr(&mut self) -> *mut u32 {
        // This guarentees the end stack pointer is 8 bytes aligned
        let align8_size = Z - (Z % 8);
        let stack_end = unsafe { self.stack.as_mut_ptr().add(align8_size) } as *mut u32;
        stack_end
    }

    pub const fn init() -> Stack<Z> {
        Stack { stack: [0; Z] }
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