use core::ptr::{self, addr_of_mut, read_volatile, write_volatile};

use alloc::{alloc::Global, boxed::Box};

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
        timeout::Timeout,
        CpuVariant,
    },
    list::singly_linked as sl,
    stdio,
};

#[cfg(feature = "kernel-debug")]
use crate::println;

use super::sync::AcquireOutcome;

pub enum SupervisorCallReason {
    Syscall(SVCCallParams),
    Interrupted,
}

pub enum SchedulerVerdict<'a, CPU: CpuVariant> {
    RunProcess(&'a Thread<'a, CPU>),
    Idle,
}

pub enum SyscallOutcome {
    // Syscall completed immediately with the given return value
    Completed(i32),
    // Syscall made the thread pending and is waiting for a signal to complete
    Pending,
}

impl From<Option<i32>> for SyscallOutcome {
    fn from(value: Option<i32>) -> Self {
        match value {
            Some(ret) => SyscallOutcome::Completed(ret),
            None => SyscallOutcome::Pending,
        }
    }
}

// requires generic_const?? ... unstable feature
pub trait KernelSpec {
    const KOBJS: u32;
}

// TODO make this value a const generic
// Define the tick duration in milliseconds
pub const MS_PER_TICK: u64 = 10;

/* This address must be accessible from asm */
#[used]
#[no_mangle]
static mut Z_SYSCALL_FLAG: u32 = 0;

// CPU: CPU variant
pub struct Kernel<'a, CPU: CpuVariant, const KOBJS: usize> {
    tasks: sl::List<'a, Thread<'a, CPU>, Runqueue>,

    // systick
    systick: SysTick,

    // Ticks counter: period: P (ms)
    ticks: u64,

    // Idle thread
    idle: Thread<'a, CPU>,

    // Kernel objects (Sync) for synchronization
    kobj: [Option<Box<dyn KernelObjectTrait<'a, CPU> + 'a>>; KOBJS],
}

impl<'a, CPU: CpuVariant, const KOBJS: usize> Kernel<'a, CPU, KOBJS> {
    pub fn init(systick: SysTick) -> Kernel<'a, CPU, KOBJS> {
        let idle = Idle::init();

        Kernel {
            tasks: sl::List::empty(),
            systick,
            ticks: 0,
            idle,
            kobj: [const { None }; KOBJS],
        }
    }

    pub fn register_thread(&mut self, thread: &'a Thread<'a, CPU>) {
        self.tasks.push_front(thread);
        thread.state.set(super::thread::ThreadState::Running);
    }

    fn increment_ticks(&mut self) {
        self.ticks += 1;
    }

    pub fn get_ticks(&self) -> u64 {
        self.ticks
    }

    pub fn kernel_loop(&mut self) {
        // Retrieve next thread to be executed
        let scheduler_verdict = self.sched_choose_next();

        match scheduler_verdict {
            // Switch to chosen user process
            // when returning from user process, we need to handle various events
            SchedulerVerdict::RunProcess(process) => match Self::switch_to_process(process) {
                SupervisorCallReason::Syscall(syscall_params) => unsafe {
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
            // can guarentee 100% it wasn't an interrupt which triggered the
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
                // sub 1 because return address (RA) includes the "thumb" flag that
                // need to be removed to get the actual instruction
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

    /// Create a new kernel object of type S with the given initialized primitive
    /// and return the index of the created object
    ///
    /// # Arguments
    ///
    /// * `initialized_sync` - The initialized synchronization primitive
    ///
    /// # Returns
    ///
    /// * The index of the created object or None if slot allocation failed
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

    fn kobj_create_default<S>(&mut self) -> Option<i32>
    where
        S: SyncPrimitive<'a, CPU> + Default + 'a,
    {
        self.kobj_create(S::default())
    }

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
            match obj_ref.acquire(thread, ticks, timeout) {
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

    /// Handle the syscall from the thread
    ///
    /// Return Some(i32) value to return to the process, or None if the syscall
    /// has not completed yet.
    ///
    /// A null (0) value returned to the process means the syscall succeeded
    ///
    /// Note that the thread must not be marked as "Running" if the returned value
    /// is not Some()
    unsafe fn do_syscall(
        &mut self,
        thread: &'a Thread<'a, CPU>,
        syscall: Syscall,
    ) -> SyscallOutcome {
        #[cfg(feature = "kernel-debug")]
        println!("{:?}", syscall);

        #[cfg(feature = "kernel-stats")]
        thread.stats.syscalls.set(thread.stats.syscalls.get() + 1);

        match syscall {
            Syscall::Kernel(KernelSyscall::Yield) => SyscallOutcome::Completed(0),
            Syscall::Kernel(KernelSyscall::Sleep { ms }) => match ms {
                0 => SyscallOutcome::Completed(0),
                u32::MAX => {
                    thread.state.set(ThreadState::Stopped);
                    SyscallOutcome::Completed(0)
                }
                _ => {
                    let expiration_time = self.get_ticks() + ms as u64 / MS_PER_TICK;

                    thread
                        .state
                        .set(ThreadState::Pending(PendingContext::new_timeout(Some(
                            expiration_time,
                        ))));

                    SyscallOutcome::Pending
                }
            },
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
            Syscall::Kernel(KernelSyscall::Fork) => {
                // 1. Allocate stack + thread
                // 2. Clone (clone) the thread
                // 3. Set stack pointer for forked thread
                // 4. Set syscall return var for forked thread
                // 5. register fork
                // 6. return from syscall

                SyscallOutcome::Completed(Kerr::NotSupported as i32)
            }
            Syscall::Io(IoSyscall::Print { ptr, len }) => {
                // rebuild &[u8] from (string and len)
                let slice = core::slice::from_raw_parts(ptr, len);

                // Direct write
                stdio::write_bytes(slice);

                SyscallOutcome::Completed(0)
            }
            _ => SyscallOutcome::Completed(Kerr::NoSuchSyscall as i32),
        }
    }

    fn handle_interrupts(&mut self) {
        // 1. Handle systick interrupt if it occured
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
