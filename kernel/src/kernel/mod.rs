use core::ffi::c_void;

pub mod errno;
pub mod kernel;
pub mod stack;
pub mod syscalls;
pub mod thread;
pub mod userspace;

// This function can be naked as it will never return !
pub type ThreadEntry = extern "C" fn(*mut c_void) -> !;

pub const fn size_of_init_stack_frame<ISF: InitStackFrameTrait>() -> usize {
    size_of::<ISF>()
}

pub trait InitStackFrameTrait: Sized {
    const SIZE_BYTES: usize = size_of_init_stack_frame::<Self>();
    const SIZE_WORDS: usize = size_of_init_stack_frame::<Self>() / 4;

    fn initialize_at(stack_ptr: *mut u32, entry: ThreadEntry, arg0: *mut c_void);
}

pub trait CpuVariant {
    type CalleeContext: Default;
    type InitStackFrame: InitStackFrameTrait;

    unsafe fn switch_to_user(
        stack_ptr: *mut u32,
        process_regs: *mut Self::CalleeContext,
    ) -> *mut u32;
}
