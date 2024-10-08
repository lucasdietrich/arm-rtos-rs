use super::{stack::Stack, CpuVariant, InitStackFrameTrait, ThreadEntry};
use crate::list::{self, Node};
use core::{cell::Cell, ffi::c_void, fmt::Display, ptr::null_mut};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    Stopped,
    Running,
    Pending,
}

#[repr(C)]
pub struct Thread<'a, CPU: CpuVariant> {
    pub stack_ptr: Cell<*mut u32>,
    pub context: Cell<CPU::CalleeContext>,
    pub state: Cell<ThreadState>,
    next: list::Link<'a, Thread<'a, CPU>>,
}

impl<'a, CPU: CpuVariant> Node<'a, Thread<'a, CPU>> for Thread<'a, CPU> {
    fn next(&'a self) -> &'a list::Link<'a, Thread<'a, CPU>> {
        &self.next
    }
}

impl<'a, CPU: CpuVariant> Thread<'a, CPU> {
    pub fn is_initialized(&self) -> bool {
        !self.stack_ptr.get().is_null()
    }

    pub fn init(stack: &Stack, entry: ThreadEntry, arg0: *mut c_void) -> Self {
        let thread = Thread {
            stack_ptr: Cell::new(unsafe { stack.stack_end.sub(CPU::InitStackFrame::SIZE_WORDS) }),
            next: list::Link::empty(),
            context: Cell::new(CPU::CalleeContext::default()),
            state: Cell::new(ThreadState::Stopped),
        };

        CPU::InitStackFrame::initialize_at(thread.stack_ptr.get(), entry, arg0);

        thread
    }
}

impl<'a, CPU: CpuVariant> Display for Thread<'a, CPU> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Thread sp=0x{:08x}", self.stack_ptr.get() as u32)
    }
}
