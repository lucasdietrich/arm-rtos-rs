use core::{ffi::c_void, fmt::Display, ptr::null_mut};

use crate::{
    cortex_m::arm::{__basic_sf, __callee_context},
    list::{self, Node},
};

// This function can be naked as it will never return !
type ThreadEntry = extern "C" fn(*mut c_void) -> !;

pub struct Stack {
    pub stack_end: *mut u32,
    pub stack_size: usize,
}

impl Stack {
    pub fn new(stack: &'static mut [u8]) -> Option<Self> {
        let stack_size = stack.len();
        let stack_end = unsafe { stack.as_mut_ptr().add(stack_size) } as *mut u32;

        // SP wouldn't be 8 B align
        if stack_end as usize % 8 != 0 {
            return None;
        }

        Some(Stack {
            stack_end,
            stack_size,
        })
    }
}

#[repr(C)]
pub struct Thread<'a> {
    pub stack_ptr: *mut u32,

    next: list::Link<'a, Thread<'a>>,
}

impl<'a> Node<'a, Thread<'a>> for Thread<'a> {
    fn next(&'a self) -> &'a list::Link<'a, Thread<'a>> {
        &self.next
    }
}

impl<'a> Thread<'a> {
    pub const fn uninit() -> Self {
        Thread {
            stack_ptr: null_mut(),
            next: list::Link::empty(),
        }
    }

    pub fn is_initialized(&self) -> bool {
        !self.stack_ptr.is_null()
    }

    pub fn init(stack: &Stack, entry: ThreadEntry, arg1: *mut c_void) -> Self {
        #[repr(C)]
        #[allow(non_camel_case_types)]
        struct InitStackFrame {
            pub context: __callee_context, // Thread switch context
            pub exc: __basic_sf,           // Exception strack frame
        }

        // TODO: Change this value to something not significant (e.g. 0x00000000)
        const UNDEFINED_MARKER: u32 = 0xAAAAAAAA;
        // TODO: Change this value to something not significant (e.g. 0x00000000)
        const LR_DEFAULT: u32 = 0xFFFFFFFF;

        const XPSR: u32 = 0x01000000; // Thumb bit to 1

        let thread = Thread {
            stack_ptr: unsafe { stack.stack_end.sub(size_of::<InitStackFrame>() >> 2) },
            next: list::Link::empty(),
        };
        let sf = thread.stack_ptr as *mut InitStackFrame;

        // Create dummy context stack frame
        unsafe {
            (*sf).context.v1 = 0;
            (*sf).context.v2 = 0;
            (*sf).context.v3 = 0;
            (*sf).context.v4 = 0;
            (*sf).context.v5 = 0;
            (*sf).context.v6 = 0;
            (*sf).context.v7 = 0;
            (*sf).context.v8 = 0;
            (*sf).context.ip = 0;
        };
        // TODO: Any problem with 8B-unaligned SP ?

        // Create exception stack frame
        unsafe {
            (*sf).exc.r0 = arg1 as u32;
            (*sf).exc.r1 = UNDEFINED_MARKER;
            (*sf).exc.r2 = UNDEFINED_MARKER;
            (*sf).exc.r3 = UNDEFINED_MARKER;
            (*sf).exc.r12 = UNDEFINED_MARKER;
            (*sf).exc.lr = LR_DEFAULT;
            (*sf).exc.pc = entry as u32; // return address: task entry function address
            (*sf).exc.xpsr = XPSR;
        };

        thread
    }
}

impl<'a> Display for Thread<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Thread sp=0x{:08x}", self.stack_ptr as u32)
    }
}
