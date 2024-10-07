use core::{cell::Cell, ffi::c_void, fmt::Display, ptr::null_mut};

use crate::{
    cortex_m::arm::{__basic_sf, __callee_context},
    list::{self, Node},
};

use super::stack::Stack;

// This function can be naked as it will never return !
type ThreadEntry = extern "C" fn(*mut c_void) -> !;

#[repr(C)]
pub struct Thread<'a> {
    pub stack_ptr: Cell<*mut u32>,

    // TODO: Review the use of the Cell here...
    pub context: Cell<__callee_context>,

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
            stack_ptr: Cell::new(null_mut()),
            next: list::Link::empty(),
            context: Cell::new(__callee_context::zeroes()),
        }
    }

    pub fn is_initialized(&self) -> bool {
        !self.stack_ptr.get().is_null()
    }

    pub fn init(stack: &Stack, entry: ThreadEntry, arg1: *mut c_void) -> Self {
        #[repr(C)]
        #[allow(non_camel_case_types)]
        struct InitStackFrame {
            pub exc: __basic_sf, // Exception strack frame
        }

        // TODO: Change this value to something not significant (e.g. 0x00000000)
        const UNDEFINED_MARKER: u32 = 0xAAAAAAAA;
        // TODO: Change this value to something not significant (e.g. 0x00000000)
        const LR_DEFAULT: u32 = 0xFFFFFFFF;

        const XPSR: u32 = 0x01000000; // Thumb bit to 1

        let thread = Thread {
            stack_ptr: Cell::new(unsafe { stack.stack_end.sub(size_of::<InitStackFrame>() >> 2) }),
            next: list::Link::empty(),
            context: Cell::new(__callee_context::default()),
        };
        let sf = thread.stack_ptr.get() as *mut InitStackFrame;

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
        write!(f, "Thread sp=0x{:08x}", self.stack_ptr.get() as u32)
    }
}
