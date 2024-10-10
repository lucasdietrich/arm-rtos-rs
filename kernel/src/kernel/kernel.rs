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
    count: usize,
    current: usize,

    // List of pending threads
    timeout_queue: list::List<'a, Thread<'a, CPU>>,

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
            count: 0, // main thread
            current: 0,
            timeout_queue: list::List::empty(),
            systick,
            ticks: 0,
            idle,
        }
    }

    pub fn register_thread(&mut self, thread: &'a Thread<'a, CPU>) {
        self.tasks.push_front(&thread);
        thread.state.set(super::thread::ThreadState::Running);
        self.count += 1;
    }

    pub fn print_tasks(&self) {
        println!("print_tasks (cur: {} count: {})", self.current, self.count);
        for task in self.tasks.iter() {
            println!("{}", task);
        }
    }

    pub fn current(&self) -> &'a Thread<'a, CPU> {
        for (index, task) in self.tasks.iter().enumerate() {
            if self.current == index {
                return task;
            }
        }
        panic!("Invalid current index");
    }

    pub fn increment_ticks(&mut self) {
        self.ticks += 1;
    }

    pub fn get_ticks(&self) -> u64 {
        self.ticks
    }

    pub fn sched_next_thread(&mut self) {
        self.current = (self.current + 1) % self.count;
    }

    fn switch_to(&mut self, current: &Thread<'_, CPU>) -> SupervisorCallReason {
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
            let syscall_flag = read_volatile(&*addr_of_mut!(Z_SYSCALL_FLAG));

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

                let r0 = ptr::read(new_process_sp.add(0));
                let r1 = ptr::read(new_process_sp.add(1));
                let r2 = ptr::read(new_process_sp.add(2));
                let r3 = ptr::read(new_process_sp.add(3));

                // "svc 0xbb" is encoded as the following 16bits instruction
                // dfbb
                let pc_svc = ptr::read(new_process_sp.add(6)) as *const u16;
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

    unsafe fn do_syscall(&mut self, thread: &'a Thread<'a, CPU>, syscall: Syscall) -> i32 {
        println!("{:?}", syscall);

        match syscall {
            Syscall::Kernel(KernelSyscall::Yield) => 0,
            Syscall::Io(IoSyscall::Print { ptr, len }) => {
                // rebuild &[u8] from (string and len)
                let slice = core::slice::from_raw_parts(ptr, len);

                // Direct write
                stdio::write_bytes(slice);

                0
            }
            _ => Kerr::ENOSYS as i32,
        }
    }

    pub fn kernel_loop(&mut self) {
        // Retrieve next thread to be executed
        let current = self.current();

        // Switch to chosen user process
        // when returning from user process, we need to handle various events
        match self.switch_to(current) {
            SupervisorCallReason::Syscall(syscall_params) => {
                unsafe {
                    let ret = if let Some(syscall) = Syscall::from_svc_params(syscall_params) {
                        self.do_syscall(current, syscall)
                    } else {
                        Kerr::ENOSYS as i32
                    };
                    // Set syscall return value at r0
                    ptr::write(current.stack_ptr.get().add(0), ret as u32);
                }
            }
            SupervisorCallReason::Interrupted => {
                // 1. Handle systick interrupt if it occured
                if self.systick.get_countflag() {
                    self.increment_ticks();
                }
            }
        }

        self.sched_next_thread();
    }
}
