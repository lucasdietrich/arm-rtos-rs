use super::{
    stack::{Stack, StackInfo},
    CpuVariant, InitStackFrameTrait, ThreadEntry,
};
use crate::list::{self, Node};
use core::{cell::Cell, ffi::c_void, fmt::Display, ptr::null_mut};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    Stopped,
    Running,
    Pending,
}

/// Represents a thread in a cooperative multitasking environment.
///
/// ## Design Details
///
/// The `Thread` struct uses `Cell` for certain fields to allow internal mutability, which
/// is essential in contexts where the thread is referenced through the linked list it
/// belongs to.
#[repr(C)]
pub struct Thread<'a, CPU: CpuVariant> {
    /// Stack pointer position for this thread when it is not actively running.
    ///
    /// This pointer is updated whenever the thread yields or is preempted,
    /// and it is restored when the thread resumes execution.
    pub stack_ptr: Cell<*mut u32>,

    /// Snapshot of CPU register states when the thread last yielded the CPU.
    ///
    /// This context includes callee-saved registers that is preserved across
    /// thread switches, allowing the thread to continue from the exact point
    /// it left off.
    pub context: Cell<CPU::CalleeContext>,

    /// Current state of the thread.
    ///
    /// Represents whether the thread is actively running, stopped, or pending
    /// in the scheduler. This state determines its availability and readiness
    /// for CPU execution.
    pub state: Cell<ThreadState>,

    /// Internal reference to the next thread in a linked list structure.
    ///
    /// This link is used to organize threads in a list.
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

    pub fn init(stack: &StackInfo, entry: ThreadEntry, arg0: *mut c_void) -> Self {
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
