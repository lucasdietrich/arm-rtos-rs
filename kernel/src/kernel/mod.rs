use core::ffi::c_void;

pub mod errno;
pub mod idle;
pub mod kernel;
pub mod stack;
pub mod sync;
pub mod syscalls;
pub mod thread;
pub mod timeout;
pub mod userspace;

// This function can be naked as it will never return !
pub type ThreadEntry = extern "C" fn(*mut c_void) -> !;

pub trait InitStackFrameTrait: Sized {
    const SIZE_BYTES: usize = size_of::<Self>();
    const SIZE_WORDS: usize = size_of::<Self>() / 4;

    fn initialize_at(stack_ptr: *mut u32, entry: ThreadEntry, arg0: *mut c_void);
}

pub trait CpuVariant {
    const FCPU: u32;

    type CalleeContext: Default + Clone + Copy; // Clone and Copy is required to fork a thread
    type InitStackFrame: InitStackFrameTrait;

    unsafe fn switch_to_user(
        stack_ptr: *mut u32,
        process_regs: *mut Self::CalleeContext,
    ) -> *mut u32;
}
