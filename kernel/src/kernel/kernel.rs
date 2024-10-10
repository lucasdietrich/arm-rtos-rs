use super::{
    idle::Idle,
    syscalls::{KernelSyscall, SVCCallParams},
    thread::Thread,
    CpuVariant,
};
use crate::{
    cortex_m::systick::SysTick,
    kernel::{
        errno::Kerr,
        syscalls::{IoSyscall, Syscall},
        thread::{PendingReason, ThreadState},
    },
    list, println, stdio,
};
use core::{
    ptr::{self, addr_of_mut, read_volatile, write_volatile},
    u64,
};

pub enum SupervisorCallReason {
    Syscall(SVCCallParams),
    Interrupted,
}

pub enum SchedulerVerdict<'a, CPU: CpuVariant> {
    RunProcess(&'a Thread<'a, CPU>),
    Idle,
}

/* This address must be accessible from asm */
#[used]
#[no_mangle]
static mut Z_SYSCALL_FLAG: u32 = 0;

// CPU: CPU variant
#[repr(C)]
pub struct Kernel<'a, CPU: CpuVariant> {
    tasks: list::List<'a, Thread<'a, CPU>>,

    // systick
    systick: SysTick,

    // Ticks counter: period: P (ms)
    ticks: u64,

    // Idle thread
    idle: Thread<'a, CPU>,
}

impl<'a, CPU: CpuVariant> Kernel<'a, CPU> {
    pub fn init(systick: SysTick) -> Kernel<'a, CPU> {
        let idle = Idle::init();

        Kernel {
            tasks: list::List::empty(),
            // timeout_queue: list::List::empty(),
            systick,
            ticks: 0,
            idle,
        }
    }

    pub fn register_thread(&mut self, thread: &'a Thread<'a, CPU>) {
        self.tasks.push_front(&thread);
        thread.state.set(super::thread::ThreadState::Running);
    }

    pub fn print_tasks(&self) {
        for task in self.tasks.iter() {
            println!("{}", task);
        }
    }

    pub fn increment_ticks(&mut self) {
        self.ticks += 1;
    }

    pub fn get_ticks(&self) -> u64 {
        self.ticks
    }

    fn switch_to(current: &Thread<'_, CPU>) -> SupervisorCallReason {
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

    /// Handle the syscall from the thread
    ///
    /// Return Some(i32) value to return to the process, or None if the syscall
    /// has not completed yet.
    ///
    /// A null (0) value returned to the process means the syscall succeeded
    ///
    /// Note that the thread must not be marked as "Running" if the returned value
    /// is not Some()
    unsafe fn do_syscall(&mut self, thread: &'a Thread<'a, CPU>, syscall: Syscall) -> Option<i32> {
        println!("{:?}", syscall);

        #[cfg(feature = "kernel-stats")]
        thread.stats.syscalls.set(thread.stats.syscalls.get() + 1);

        match syscall {
            Syscall::Kernel(KernelSyscall::Yield) => Some(0),
            Syscall::Kernel(KernelSyscall::Sleep { ms }) => match ms {
                0 => Some(0),
                u32::MAX => {
                    thread.state.set(ThreadState::Stopped);
                    Some(0)
                }
                _ => {
                    // TODO replace 10 with the actual TICKS_PER_MSEC value
                    let expiration_time = self.get_ticks() + (ms / 10) as u64;

                    thread
                        .state
                        .set(ThreadState::Pending(PendingReason::Timeout(
                            expiration_time,
                        )));

                    None
                }
            },
            Syscall::Io(IoSyscall::Print { ptr, len }) => {
                // rebuild &[u8] from (string and len)
                let slice = core::slice::from_raw_parts(ptr, len);

                // Direct write
                stdio::write_bytes(slice);

                Some(0)
            }
            _ => Some(Kerr::ENOSYS as i32),
        }
    }

    fn sched_choose_next(&mut self) -> SchedulerVerdict<'a, CPU> {
        // Order by priority + roll over available thread if there are multiple
        let mut thread_candidate: Option<&Thread<'a, CPU>> = None;

        for (index, thread) in self
            .tasks
            .iter()
            .filter(|thread| thread.is_ready())
            .enumerate()
        {
            return SchedulerVerdict::RunProcess(thread);
        }

        SchedulerVerdict::Idle
    }

    fn handle_interrupts(&mut self) {
        // 1. Handle systick interrupt if it occured
        if self.systick.get_countflag() {
            self.increment_ticks();

            // Check if any thread timed out
            for thread in self
                .tasks
                .iter()
                .filter(|thread| thread.get_timeout_ticks().is_some())
            {
                if self.get_ticks() >= thread.get_timeout_ticks().unwrap() {
                    thread.set_ready();
                    unsafe {
                        thread.set_syscall_return_value_unchecked(-(Kerr::ETIMEDOUT as i32));
                    }
                }
            }
        }
    }

    pub fn kernel_loop(&mut self) {
        // Retrieve next thread to be executed
        let scheduler_verdict = self.sched_choose_next();

        match scheduler_verdict {
            // Switch to chosen user process
            // when returning from user process, we need to handle various events
            SchedulerVerdict::RunProcess(process) => match Self::switch_to(process) {
                SupervisorCallReason::Syscall(syscall_params) => unsafe {
                    let ret = if let Some(syscall) = Syscall::from_svc_params(syscall_params) {
                        self.do_syscall(process, syscall)
                    } else {
                        Some(Kerr::ENOSYS as i32)
                    };

                    // Syscall completed, return value in user process stack in r0 register
                    if let Some(result) = ret {
                        process.set_syscall_return_value_unchecked(result);
                    }
                },
                SupervisorCallReason::Interrupted => {
                    self.handle_interrupts();

                    // TODO: If current thread is cooperative, we must return to it
                }
            },

            SchedulerVerdict::Idle => match Self::switch_to(&self.idle) {
                SupervisorCallReason::Interrupted => self.handle_interrupts(),
                // Idle thread should never use syscalls
                _ => panic!("IDLE fired syscall"),
            },
        };
    }
}
