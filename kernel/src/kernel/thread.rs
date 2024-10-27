use super::{
    errno::Kerr,
    stack::StackInfo,
    sync::{SwapData, Swappable},
    timeout::TimeoutInstant,
    CpuVariant, InitStackFrameTrait, ThreadEntry,
};
use crate::list::{self, singly_linked as sl};
use core::{cell::Cell, cmp::Ordering, ffi::c_void, fmt::Display, ptr, time};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    Stopped,
    Running,
    Pending(PendingContext),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PendingContext {
    sync_kobj_index: Option<u32>,
    timeout_instant: Option<TimeoutInstant>,
}

pub enum UnpendReason {
    Timeout,
    Swap(SwapData),
}

pub struct Runqueue;
impl list::Marker for Runqueue {}
pub struct Waitqueue;
impl list::Marker for Waitqueue {}

impl PendingContext {
    pub fn new_sync(
        sync_kobj_index: u32,
        timeout_instant: Option<TimeoutInstant>,
    ) -> PendingContext {
        PendingContext {
            sync_kobj_index: Some(sync_kobj_index),
            timeout_instant,
        }
    }

    pub fn new_timeout(timeout_instant: TimeoutInstant) -> PendingContext {
        PendingContext {
            sync_kobj_index: None,
            timeout_instant: Some(timeout_instant),
        }
    }

    pub fn get_timeout(&self) -> Option<TimeoutInstant> {
        self.timeout_instant
    }
}

#[cfg(feature = "kernel-stats")]
#[derive(Default)]
pub struct ThreadStats {
    pub(super) syscalls: Cell<u32>,
}

// Thread priority model is the same as Zephyr RTOS:
// read: <https://docs.zephyrproject.org/latest/kernel/services/threads/index.html#id12>
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

impl Ord for ThreadPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        other.raw_priority().cmp(&self.raw_priority())
    }
}

impl PartialOrd for ThreadPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents a thread in a multitasking environment.
///
/// ## Design Details
///
/// The `Thread` struct uses `Cell` for certain fields to allow internal mutability, which
/// is essential in contexts where the thread is referenced through the linked list it
/// belongs to.
pub struct Thread<'a, CPU: CpuVariant> {
    /// Stack pointer position for this thread when it is not actively running.
    ///
    /// This pointer is updated whenever the thread yields or is preempted,
    /// and it is restored when the thread resumes execution.
    pub(super) stack_ptr: Cell<*mut u32>,

    /// Snapshot of CPU register states when the thread last yielded the CPU.
    ///
    /// This context includes callee-saved registers that is preserved across
    /// thread switches, allowing the thread to continue from the exact point
    /// it left off.
    pub(super) context: Cell<CPU::CalleeContext>,

    /// Current state of the thread.
    ///
    /// Represents whether the thread is actively running, stopped, or pending
    /// in the scheduler. This state determines its availability and readiness
    /// for CPU execution.
    pub(super) state: Cell<ThreadState>,

    /// Thread priority (preemptive/cooperative)
    pub priority: ThreadPriority,

    /// This link is used to organize threads in kernel list of known threads
    runqueue_next: sl::Link<'a, Thread<'a, CPU>, Runqueue>,

    /// This link is used to make the thread waiting for a synchronization object
    /// by adding it to the queue of waiting thread for the object.
    waitqueue_next: sl::Link<'a, Thread<'a, CPU>, Waitqueue>,

    /// Stats for the current thread
    #[cfg(feature = "kernel-stats")]
    pub stats: ThreadStats,
}

impl<'a, CPU: CpuVariant> sl::Node<'a, Thread<'a, CPU>, Runqueue> for Thread<'a, CPU> {
    fn next(&'a self) -> &'a sl::Link<'a, Thread<'a, CPU>, Runqueue> {
        &self.runqueue_next
    }
}

impl<'a, CPU: CpuVariant> sl::Node<'a, Thread<'a, CPU>, Waitqueue> for Thread<'a, CPU> {
    fn next(&'a self) -> &'a sl::Link<'a, Thread<'a, CPU>, Waitqueue> {
        &self.waitqueue_next
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
            context: Cell::new(CPU::CalleeContext::default()),
            priority: ThreadPriority::from(raw_priority),
            state: Cell::new(ThreadState::Stopped),
            runqueue_next: sl::Link::empty(),
            waitqueue_next: sl::Link::empty(),
            #[cfg(feature = "kernel-stats")]
            stats: ThreadStats::default(),
        };

        CPU::InitStackFrame::initialize_at(thread.stack_ptr.get(), entry, arg0);

        thread
    }

    pub fn set_ready(&self) {
        self.state.set(ThreadState::Running);
    }

    pub fn set_pending(&self, sync: u32, timeout_instant: TimeoutInstant) {
        self.state
            .set(ThreadState::Pending(PendingContext::new_sync(
                sync,
                Some(timeout_instant),
            )));
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.state.get(), ThreadState::Running)
    }

    pub fn is_preemptable(&self) -> bool {
        matches!(self.priority, ThreadPriority::Preemptive(..))
    }

    // Return time (in ticks) when the thread is schedulded for timeout
    pub fn get_timeout_instant(&self) -> Option<TimeoutInstant> {
        match self.state.get() {
            ThreadState::Pending(reason) => reason.get_timeout(),
            _ => None,
        }
    }

    /// Determines if the thread has exceeded its scheduled timeout based on system ticks.
    ///
    /// # Arguments
    /// * `sys_ticks` - The current system ticks count to compare against the timeout.
    ///
    /// # Returns
    /// * `true` - If the timeout has expired
    /// * `false` - If the timeout is still in the future or if no timeout is scheduled
    pub fn has_timed_out(&self, sys_ticks: u64) -> bool {
        self.get_timeout_instant()
            .map(|timeout| timeout.is_past(sys_ticks))
            .unwrap_or(false)
    }

    pub fn lives_in_waitqueue(&self) -> Option<u32> {
        match self.state.get() {
            ThreadState::Pending(PendingContext {
                sync_kobj_index: Some(sync_kobj_index),
                ..
            }) => Some(sync_kobj_index),
            _ => None,
        }
    }

    pub fn set_syscall_return_value(&self, _ret: i32) {
        // Make sure a syscall is pending, otherwise it could break the stack
        todo!()
    }

    // make sure a syscall is pending, otherwise it could breack the stack
    pub unsafe fn set_syscall_return_value_unchecked(&self, ret: i32) {
        ptr::write(self.stack_ptr.get().add(0), ret as u32);
    }

    pub fn unpend<S: Swappable>(&self, swap: &S) {
        self.set_ready();
        unsafe {
            self.set_syscall_return_value_unchecked(swap.to_syscall_ret());
        }
    }

    pub fn unpend_timeout(&self) {
        self.set_ready();
        unsafe {
            self.set_syscall_return_value_unchecked(Kerr::TimedOut as i32);
        }
    }
}

impl<'a, CPU: CpuVariant> Display for Thread<'a, CPU> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Thread sp=0x{:08x}", self.stack_ptr.get() as u32)
    }
}
