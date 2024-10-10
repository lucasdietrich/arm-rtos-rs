use super::{
    stack::{Stack, StackInfo},
    CpuVariant, InitStackFrameTrait, ThreadEntry,
};
use crate::list::{self, Node};
use core::{
    cell::Cell,
    ffi::c_void,
    fmt::Display,
    future::Pending,
    ptr::{self, null_mut},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    Stopped,
    Running,
    Pending(PendingReason),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PendingReason {
    // waiting for timeout, wait until specified uptime is reached (in ticks)
    Timeout(u64),
    // Waiting for synchronization object to become ready
    Sync(i32),
    // Waiting for synchronization object to become ready or timeout
    SyncOrTimeout(i32, u64),
}

#[cfg(feature = "kernel-stats")]
#[derive(Default)]
pub struct ThreadStats {
    pub syscalls: Cell<u32>,
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThreadPriority {
    Cooperative(i8),
    Preemptive(i8),
}

impl ThreadPriority {
    pub fn from(priority: i8) -> ThreadPriority {
        if priority >= 0 {
            ThreadPriority::Preemptive(priority)
        } else {
            ThreadPriority::Cooperative(priority)
        }
    }

    pub fn raw_priority(&self) -> i8 {
        match self {
            ThreadPriority::Preemptive(priority) => *priority,
            ThreadPriority::Cooperative(priority) => *priority,
        }
    }
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

    /// Thread priority (preemptive/cooperative)
    pub priority: ThreadPriority,

    /// Stats for the current thread
    #[cfg(feature = "kernel-stats")]
    pub stats: ThreadStats,

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

    pub fn init(
        stack: &StackInfo,
        entry: ThreadEntry,
        arg0: *mut c_void,
        raw_priority: i8,
    ) -> Self {
        let thread = Thread {
            stack_ptr: Cell::new(unsafe { stack.stack_end.sub(CPU::InitStackFrame::SIZE_WORDS) }),
            next: list::Link::empty(),
            context: Cell::new(CPU::CalleeContext::default()),
            priority: ThreadPriority::from(raw_priority),
            state: Cell::new(ThreadState::Stopped),
            #[cfg(feature = "kernel-stats")]
            stats: ThreadStats::default(),
        };

        CPU::InitStackFrame::initialize_at(thread.stack_ptr.get(), entry, arg0);

        thread
    }

    pub fn set_ready(&self) {
        self.state.set(ThreadState::Running);
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.state.get(), ThreadState::Running)
    }

    // Return time (in ticks) when the thread is schedulded for timeout
    pub fn get_timeout_ticks(&self) -> Option<u64> {
        match self.state.get() {
            ThreadState::Pending(PendingReason::SyncOrTimeout(.., timeout)) => Some(timeout),
            ThreadState::Pending(PendingReason::Timeout(timeout)) => Some(timeout),
            _ => None,
        }
    }

    pub fn set_syscall_return_value(&self, ret: i32) {
        todo!()
    }

    // make sure a syscall is pending, otherwise it could breack the stack
    pub unsafe fn set_syscall_return_value_unchecked(&self, ret: i32) {
        ptr::write(self.stack_ptr.get().add(0), ret as u32);
    }
}

impl<'a, CPU: CpuVariant> Display for Thread<'a, CPU> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Thread sp=0x{:08x}", self.stack_ptr.get() as u32)
    }
}
