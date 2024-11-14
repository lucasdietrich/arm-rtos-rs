//! Kernel module for the embedded operating system.
//!
//! This module contains the core implementation of the kernel, including the scheduler, syscall handling,
//! thread management, and synchronization primitives.

use core::{
    alloc::{self, Allocator},
    ptr::{self, addr_of_mut, read_volatile, write_volatile, NonNull},
};

use ::alloc::{alloc::Global, boxed::Box};

use crate::{
    cortex_m::systick::SysTick,
    kernel::{
        errno::Kerr,
        idle::Idle,
        sync::{
            KernelObject, KernelObjectTrait, Mutex, Semaphore, Signal, SignalValue, SwapData, Sync,
            SyncPrimitive,
        },
        syscalls::{
            IoSyscall, KernelSyscall, SVCCallParams, SyncPrimitiveCreate, SyncPrimitiveType,
            Syscall,
        },
        thread::{PendingContext, Runqueue, Thread, ThreadState},
        timeout::{Timeout, TimeoutInstant},
        CpuVariant,
    },
    list::singly_linked as sl,
    println, stdio,
};

#[cfg(feature = "kernel-debug")]
use crate::println;

use super::sync::AcquireOutcome;

pub const USER_MALLOC_DEFAULT_ALIGN: usize = 4;
pub const USER_MALLOC_MIN_ALIGN: usize = 2;

/// Represents the reason why the supervisor (kernel) was called.
///
/// This enum is used to distinguish between different reasons for entering the kernel from user mode,
/// such as a syscall or an interrupt.
pub enum SupervisorCallReason {
    /// A syscall was invoked by the user process.
    Syscall(SVCCallParams),
    /// An interrupt occurred.
    Interrupted,
}

/// The result of the scheduler's decision on which process to run next.
pub enum SchedulerVerdict<'a, CPU: CpuVariant> {
    /// Run the specified process (thread).
    RunProcess(&'a Thread<'a, CPU>),
    /// No ready process to run; enter idle mode.
    Idle,
}

/// Represents the outcome of a syscall execution.
pub enum SyscallOutcome {
    /// Syscall completed immediately with the given return value.
    Completed(i32),
    /// Syscall made the thread pending and is waiting for a signal to complete.
    Pending,
    /// 2 bytes align raw pointer
    RawPtr(*const u8),
}

impl From<Option<i32>> for SyscallOutcome {
    /// Converts an `Option<i32>` into a `SyscallOutcome`.
    ///
    /// If the option is `Some(ret)`, returns `SyscallOutcome::Completed(ret)`.
    /// If the option is `None`, returns `SyscallOutcome::Pending`.
    fn from(value: Option<i32>) -> Self {
        match value {
            Some(ret) => SyscallOutcome::Completed(ret),
            None => SyscallOutcome::Pending,
        }
    }
}

/// A flag used to indicate if a syscall has been invoked by a user process.
///
/// This address must be accessible from assembly code.
#[used]
#[no_mangle]
static mut Z_SYSCALL_FLAG: u32 = 0;

/// The core kernel structure that manages threads, scheduling, and synchronization.
///
/// # Type Parameters
///
/// * `'a` - The lifetime associated with the kernel and its components.
/// * `CPU` - The CPU variant, implementing the `CpuVariant` trait.
/// * `K` - The maximum number of kernel objects (synchronization primitives).
/// * `F` - The frequency of the system tick (SysTick) in Hz.
pub struct Kernel<'a, CPU: CpuVariant, const K: usize, const F: u32> {
    /// The list of tasks (threads) managed by the kernel.
    tasks: sl::List<'a, Thread<'a, CPU>, Runqueue>,

    /// The system tick timer.
    systick: SysTick<F>,

    /// The system tick counter.
    ticks: u64,

    /// The idle thread.
    idle: Thread<'a, CPU>,

    /// The array of kernel objects (synchronization primitives).
    kobj: [Option<Box<dyn KernelObjectTrait<'a, CPU> + 'a>>; K],
}

impl<'a, CPU: CpuVariant, const K: usize, const F: u32> Kernel<'a, CPU, K, F> {
    /// Initializes a new kernel instance.
    ///
    /// # Arguments
    ///
    /// * `systick` - The system tick timer.
    ///
    /// # Returns
    ///
    /// A new instance of the kernel.
    pub fn init(systick: SysTick<F>) -> Kernel<'a, CPU, K, F> {
        let idle = Idle::init();

        Kernel {
            tasks: sl::List::empty(),
            systick,
            ticks: 0,
            idle,
            kobj: [const { None }; K],
        }
    }

    /// Converts milliseconds to system ticks based on the system tick frequency.
    ///
    /// # Arguments
    ///
    /// * `ms` - The duration in milliseconds.
    ///
    /// # Returns
    ///
    /// The equivalent duration in system ticks.
    pub fn ms_to_ticks(ms: u32) -> u64 {
        ms as u64 * F as u64 / 1000
    }

    /// Registers a new thread with the kernel and marks it as ready to run.
    ///
    /// # Arguments
    ///
    /// * `thread` - A reference to the thread to register.
    pub fn register_thread(&mut self, thread: &'a Thread<'a, CPU>) {
        self.tasks.push_front(thread);
        thread.state.set(super::thread::ThreadState::Running);
    }

    /// Increments the system tick counter by one.
    fn increment_ticks(&mut self) {
        self.ticks += 1;
    }

    /// Retrieves the current value of the system tick counter.
    ///
    /// # Returns
    ///
    /// The current system tick count.
    pub fn get_ticks(&self) -> u64 {
        self.ticks
    }

    /// The main kernel loop that handles scheduling and dispatching threads.
    ///
    /// This function selects the next thread to run, switches context to it, and handles any
    /// syscalls or interrupts that occur during its execution.
    pub fn kernel_loop(&mut self) {
        // Retrieve next thread to be executed
        let scheduler_verdict = self.sched_choose_next();

        match scheduler_verdict {
            // Switch to chosen user process
            // when returning from user process, we need to handle various events
            SchedulerVerdict::RunProcess(process) => match Self::switch_to_process(process) {
                SupervisorCallReason::Syscall(syscall_params) => unsafe {
                    #[cfg(feature = "kernel-debug")]
                    println!("process: {:#x?}", process.context.get());

                    let ret = if let Some(syscall) = Syscall::from_svc_params(syscall_params) {
                        self.do_syscall(process, syscall)
                    } else {
                        SyscallOutcome::Completed(Kerr::NoSuchSyscall as i32)
                    };

                    // Syscall completed, return value in user process stack in r0 register
                    if let SyscallOutcome::Completed(result) = ret {
                        process.set_syscall_return_value_unchecked(result);
                    }
                },
                SupervisorCallReason::Interrupted => {
                    self.handle_interrupts();

                    // TODO: If current thread is cooperative, we must return to it
                }
            },

            SchedulerVerdict::Idle => match Self::switch_to_process(&self.idle) {
                SupervisorCallReason::Interrupted => self.handle_interrupts(),
                // Idle thread should never use syscalls
                _ => panic!("IDLE fired syscall"),
            },
        };
    }

    /// Chooses the next thread to run based on scheduling policy.
    ///
    /// This scheduler picks the ready thread with the highest priority. If no threads are ready,
    /// it returns `SchedulerVerdict::Idle` to indicate the idle thread should run.
    ///
    /// # Returns
    ///
    /// A `SchedulerVerdict` indicating the next action for the scheduler.
    fn sched_choose_next(&mut self) -> SchedulerVerdict<'a, CPU> {
        // Pick any ready thread with maximum priority
        // This naive scheduler may always pick the same thread even if other
        // threads of the same priority are ready. This can be improve by
        // defining per-thread time slice and sharing CPU time using round-robin
        // algorithm. This won't be implemented here.
        let thread_candidate = self
            .tasks
            .iter()
            .filter(|thread| thread.is_ready())
            .max_by_key(|thread| thread.priority);

        match thread_candidate {
            Some(candidate) => SchedulerVerdict::RunProcess(candidate),
            None => SchedulerVerdict::Idle,
        }
    }

    /// Switches context to the given thread (process) and returns the reason for returning to the kernel.
    ///
    /// This function saves the current process's state, switches context to the user process,
    /// and upon return, determines whether the return was due to a syscall or an interrupt.
    ///
    /// # Arguments
    ///
    /// * `current` - A reference to the thread to switch to.
    ///
    /// # Returns
    ///
    /// A `SupervisorCallReason` indicating whether a syscall was invoked or an interrupt occurred.
    fn switch_to_process(current: &Thread<'_, CPU>) -> SupervisorCallReason {
        // Retrieve process last position of stack pointer
        let process_sp = current.stack_ptr.get();

        // Retrieve process last context
        let process_context = current.context.as_ptr();

        // Switch to user process
        let new_process_sp = unsafe { CPU::switch_to_user(process_sp, process_context) };

        // At this point we returned from the user process,
        // process context has already been saved but we need
        // to save the position of the stack pointer for next execution
        current.stack_ptr.set(new_process_sp);

        unsafe {
            // If the flag is set it means, the current process called a syscall,
            // otherwise the switch was triggered by an interrupt
            //
            // Another idea to achieve this goal could have been to read the user
            // process yielded instruction and compare it to "svc" to know if the
            // user thread triggered a syscall. Unfortunately, I'm not sure we
            // can guarantee 100% it wasn't an interrupt which triggered the
            // syscall at this exact moment ???
            let syscall_flag = read_volatile(&*addr_of_mut!(Z_SYSCALL_FLAG));

            // Clear flag
            write_volatile(&mut *addr_of_mut!(Z_SYSCALL_FLAG), 0);

            if syscall_flag != 0 {
                // At this point, the process exception frame looks like this
                // sp + 00: r0 (syscall arg 0)
                // sp + 04: r1 (syscall arg 1)
                // sp + 08: r2 (syscall arg 2)
                // sp + 0C: r3 (syscall arg 3)
                // sp + 10: r12
                // sp + 14: lr
                // sp + 18: return address (instruction following the svc)
                // sp + 1C: xPSR

                // Read syscall arguments from stack
                let r0 = ptr::read(new_process_sp.add(0));
                let r1 = ptr::read(new_process_sp.add(1));
                let r2 = ptr::read(new_process_sp.add(2));
                let r3 = ptr::read(new_process_sp.add(3));

                // Read syscall main id from yielded PC
                // "svc 0xbb" is encoded as the following 16bits instruction: 0xdfbb
                let pc_svc = ptr::read(new_process_sp.add(6)) as *const u16;
                // Subtract 1 because return address (RA) includes the "thumb" flag that
                // needs to be removed to get the actual instruction
                let svc_instruction = ptr::read(pc_svc.sub(1));
                let syscall_id = (svc_instruction & 0xFF) as u8;

                let syscall_params = SVCCallParams {
                    r0,
                    r1,
                    r2,
                    r3,
                    syscall_id,
                };

                SupervisorCallReason::Syscall(syscall_params)
            } else {
                SupervisorCallReason::Interrupted
            }
        }
    }

    /// Creates a new kernel object with the given initialized synchronization primitive.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The type of the synchronization primitive.
    ///
    /// # Arguments
    ///
    /// * `initialized_sync` - The initialized synchronization primitive to wrap in a kernel object.
    ///
    /// # Returns
    ///
    /// An `Option<i32>` containing the index of the created kernel object, or `None` if allocation failed.
    fn kobj_create<S>(&mut self, initialized_sync: S) -> Option<i32>
    where
        S: SyncPrimitive<'a, CPU> + 'a,
    {
        self.kobj
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_none())
            .and_then(|(index, slot)| {
                Box::<KernelObject<'a, S, CPU>, Global>::try_new(KernelObject::new(
                    index as u32,
                    initialized_sync,
                ))
                .map(|kobj| {
                    *slot = Some(kobj);
                    index as i32
                })
                .ok()
            })
    }

    /// Creates a new kernel object with a default-initialized synchronization primitive.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The type of the synchronization primitive, which must implement `Default`.
    ///
    /// # Returns
    ///
    /// An `Option<i32>` containing the index of the created kernel object, or `None` if allocation failed.
    fn kobj_create_default<S>(&mut self) -> Option<i32>
    where
        S: SyncPrimitive<'a, CPU> + Default + 'a,
    {
        self.kobj_create(S::default())
    }

    /// Attempts to acquire a kernel object (synchronization primitive) for the given thread.
    ///
    /// # Arguments
    ///
    /// * `kobj` - The index of the kernel object to acquire.
    /// * `thread` - A reference to the thread attempting to acquire the object.
    /// * `timeout` - The timeout for the acquisition attempt.
    ///
    /// # Returns
    ///
    /// A `SyscallOutcome` indicating the result of the acquisition attempt.
    fn kobj_acquire(
        &mut self,
        kobj: i32,
        thread: &'a Thread<'a, CPU>,
        timeout: Timeout,
    ) -> SyscallOutcome {
        let ticks = self.get_ticks();
        if let Some(obj_ref) = self
            .kobj
            .get_mut(kobj as usize)
            .and_then(|slot| slot.as_mut())
        {
            // Calculate the instant when the thread should be woken up
            let timeout_instant = match timeout {
                Timeout::Forever => TimeoutInstant::new_never(),
                Timeout::Duration(ms) => TimeoutInstant::new_at(ticks + Self::ms_to_ticks(ms)),
            };

            match obj_ref.acquire(thread, timeout_instant) {
                AcquireOutcome::Obtained(swap_data) => {
                    SyscallOutcome::Completed(swap_data.to_syscall_ret())
                }
                AcquireOutcome::NotObtained => SyscallOutcome::Completed(Kerr::TryAgain as i32),
                AcquireOutcome::Pending => SyscallOutcome::Pending,
            }
        } else {
            // Invalid kernel object
            SyscallOutcome::Completed(Kerr::NoEntry as i32)
        }
    }

    /// Releases a kernel object and notifies any waiting threads.
    ///
    /// # Arguments
    ///
    /// * `kobj` - The index of the kernel object to release.
    /// * `swap_data` - The data to swap with the kernel object (if applicable).
    ///
    /// # Returns
    ///
    /// A `SyscallOutcome` indicating the result of the release operation.
    fn kobj_release_notify(&mut self, kobj: i32, swap_data: SwapData) -> SyscallOutcome {
        let ret = if let Some(obj_ref) = self
            .kobj
            .get_mut(kobj as usize)
            .and_then(|slot| slot.as_mut())
        {
            match obj_ref.release(swap_data) {
                Ok(_) => Kerr::Success,
                Err(_) => Kerr::NotSupported,
            }
        } else {
            // Invalid kernel object
            Kerr::NoEntry
        };

        SyscallOutcome::Completed(ret as i32)
    }

    /// Handles a syscall from the given thread.
    ///
    /// Executes the syscall and returns the outcome.
    ///
    /// # Arguments
    ///
    /// * `thread` - The thread that invoked the syscall.
    /// * `syscall` - The syscall to execute.
    ///
    /// # Returns
    ///
    /// A `SyscallOutcome` indicating the result of the syscall execution.
    unsafe fn do_syscall(
        &mut self,
        thread: &'a Thread<'a, CPU>,
        syscall: Syscall,
    ) -> SyscallOutcome {
        #[cfg(feature = "kernel-debug-syscalls")]
        println!("{:?}", syscall);

        #[cfg(feature = "kernel-stats")]
        thread.stats.syscalls.set(thread.stats.syscalls.get() + 1);

        match syscall {
            Syscall::Test { r0, r1, r2, r3 } => {
                // #[cfg(feature = "kernel-debug")]
                println!(
                    "Test syscall: r0={:x}, r1={:x}, r2={:x}, r3={:x}",
                    r0, r1, r2, r3
                );
                SyscallOutcome::Completed(0)
            }
            Syscall::Kernel(KernelSyscall::Yield) => SyscallOutcome::Completed(0),
            Syscall::Kernel(KernelSyscall::Sleep { ms }) => {
                let timeout = Timeout::from(ms);
                match timeout {
                    Timeout::Forever => {
                        thread.state.set(ThreadState::Stopped);
                        SyscallOutcome::Completed(0)
                    }
                    Timeout::Duration(0) => SyscallOutcome::Completed(0),
                    Timeout::Duration(ms) => {
                        thread
                            .state
                            .set(ThreadState::Pending(PendingContext::new_timeout(
                                TimeoutInstant::new_at(self.get_ticks() + Self::ms_to_ticks(ms)),
                            )));

                        SyscallOutcome::Pending
                    }
                }
            }
            Syscall::Kernel(KernelSyscall::SyncCreate { prim }) => SyscallOutcome::Completed(
                match prim {
                    SyncPrimitiveCreate::Sync => self.kobj_create_default::<Sync>(),
                    SyncPrimitiveCreate::Signal => self.kobj_create_default::<Signal>(),
                    SyncPrimitiveCreate::Mutex => self.kobj_create_default::<Mutex<'a, CPU>>(),
                    SyncPrimitiveCreate::Semaphore { init, max } => {
                        self.kobj_create(Semaphore::new(init, max))
                    }
                }
                .unwrap_or(Kerr::NoMemory as i32),
            ),
            Syscall::Kernel(KernelSyscall::Pend {
                prim: _, // sync_prim_type
                kobj,
                timeout,
            }) => self.kobj_acquire(kobj, thread, timeout),
            Syscall::Kernel(KernelSyscall::Sync { arg, prim, kobj }) => {
                let swap_data = match prim {
                    SyncPrimitiveType::Sync => SwapData::Empty,
                    SyncPrimitiveType::Signal => SwapData::Signal(SignalValue::new(arg)),
                    SyncPrimitiveType::Semaphore => SwapData::Empty,
                    SyncPrimitiveType::Mutex => SwapData::Ownership,
                };
                self.kobj_release_notify(kobj, swap_data)
            }
            Syscall::Kernel(KernelSyscall::Stop) => {
                thread.state.set(ThreadState::Stopped);
                SyscallOutcome::Completed(0)
            }
            Syscall::Kernel(KernelSyscall::MemoryAlloc { size, mut align }) => {
                // If align is 0, use default alignment
                if align == 0 {
                    align = USER_MALLOC_DEFAULT_ALIGN;
                } else if align >= USER_MALLOC_MIN_ALIGN {
                    return SyscallOutcome::Completed(Kerr::InvalidArguments as i32);
                }

                // For alignments greater than or equal to 2, the returned memory pointer
                // can safely be shifted right by one bit (ptr >> 1), as the least significant
                // bit of the aligned pointer will always be zero.
                match alloc::Layout::from_size_align(size, align) {
                    Ok(memory_layout) => match Global.allocate_zeroed(memory_layout) {
                        Ok(ptr) => {
                            let ptr = ptr.as_ptr() as *const u8 as u32;
                            let shifted_ptr = ptr >> 1;
                            SyscallOutcome::Completed(shifted_ptr as i32)
                        }
                        Err(_) => SyscallOutcome::Completed(Kerr::NoMemory as i32),
                    },
                    Err(_) => SyscallOutcome::Completed(Kerr::InvalidArguments as i32),
                }
            }
            Syscall::Kernel(KernelSyscall::MemoryFree { ptr }) => {
                if let Some(_ptr) = NonNull::new(ptr) {
                    // TODO
                    // Global.deallocate(ptr, layout) requires the original layout
                    // how to rebuild layout from ptr ?

                    SyscallOutcome::Completed(Kerr::NotSupported as i32)
                } else {
                    SyscallOutcome::Completed(Kerr::InvalidArguments as i32)
                }
            }
            Syscall::Kernel(KernelSyscall::Fork) => {
                // Needs MMU support
                // 1. Allocate stack + thread
                // 2. Clone the thread
                // 3. Set stack pointer for forked thread
                // 4. Set syscall return var for forked thread
                // 5. Register forked thread
                // 6. Return from syscall

                SyscallOutcome::Completed(Kerr::NotSupported as i32)
            }
            Syscall::Io(IoSyscall::Print { ptr, len, newline }) => {
                // Rebuild &[u8] from (string and len)
                let slice = core::slice::from_raw_parts(ptr, len);

                // Direct write
                stdio::write_bytes(slice);

                if newline {
                    stdio::write_bytes(b"\n");
                }

                SyscallOutcome::Completed(0)
            }
            Syscall::Io(IoSyscall::HexPrint { ptr, len }) => {
                // Rebuild &[u8] from (string and len)
                let slice = core::slice::from_raw_parts(ptr, len);

                stdio::write_hex(slice);

                SyscallOutcome::Completed(0)
            }
            Syscall::Io(IoSyscall::Read1) => match stdio::read() {
                Some(byte) => SyscallOutcome::Completed(byte as i32),
                None => SyscallOutcome::Completed(Kerr::TryAgain as i32),
            },
            _ => SyscallOutcome::Completed(Kerr::NoSuchSyscall as i32),
        }
    }

    /// Handles any pending interrupts, such as the system tick interrupt.
    ///
    /// This function checks for interrupts that have occurred and updates the kernel's state
    /// accordingly, such as incrementing the tick counter and managing timed-out threads.
    fn handle_interrupts(&mut self) {
        // 1. Handle systick interrupt if it occurred
        if self.systick.get_countflag() {
            self.increment_ticks();

            let sys_ticks = self.get_ticks();

            // Check if any thread timed out
            for thread in self
                .tasks
                .iter()
                .filter(|thread| thread.has_timed_out(sys_ticks))
            {
                // Remove the thread from the kobj waitqueue
                if let Some(kobj_index) = thread.lives_in_waitqueue() {
                    if let Some(kobj) = self
                        .kobj
                        .get_mut(kobj_index as usize)
                        .and_then(|obj_ref| obj_ref.as_mut())
                    {
                        kobj.remove_thread(thread)
                    }
                }

                thread.unpend_timeout();
            }
        }
    }
}
