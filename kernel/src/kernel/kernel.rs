use core::arch::global_asm;

use crate::{
    cortex_m::{
        critical_section::{Cs, GlobalIrq},
        interrupts::{self, atomic_restore, atomic_section},
    },
    list, println,
};

use super::threading::Thread;

pub fn sleep(ms: u32) {}

#[link_section = ".bss"]
#[used]
pub static mut BSS_MYVAR: u32 = 0;

#[used]
#[no_mangle]
pub static mut z_current: *mut Thread = core::ptr::null_mut();

#[used]
#[no_mangle]
pub static mut z_next: *mut Thread = core::ptr::null_mut();

// 1. Calls to pendsv saves:
//  r0-r3, r12, lr, return addr, xpsr
global_asm!(
    "
    .section .text, \"ax\"
    .global z_pendsv
    .thumb_func
z_pendsv:
    // retrieve address of current thread and save it in r0
    ldr r2, =z_current
    ldr r0, [r2]
    ldr r3, =z_next // use z_kernel offsets
    ldr r1, [r3]

_thread_switch:
    // save 'from' thread context
    push {{v1-v8, ip}}

    // save sp to 'from' thread
    str sp, [r0]

    // load sp from 'from' thread
    ldr sp, [r1]

    // restore 'to' thread context
    pop {{v1-v8, ip}}

    // load KERNEL structure into r0
    bx lr
    "
);

extern "C" {
    pub fn z_pendsv();
}

// N: Maximum number of threads supported
// F: systick frequency (Hz)
#[repr(C)]
pub struct Kernel<'a, const F: u32 = 1> {
    tasks: list::List<'a, Thread<'a>>,
    count: usize,
    current: usize,

    // Ticks counter: period: P (ms)
    ticks: u64,
}

static mut MAIN_THREAD: Thread<'static> = Thread::uninit();

impl<const F: u32> Kernel<'static, F> {
    pub const fn init() -> Kernel<'static, F> {
        Kernel {
            tasks: list::List::empty(),
            count: 0, // main thread
            current: 0,
            ticks: 0,
        }
    }

    pub fn register_main_thread(&mut self) {
        let main_thread = unsafe { &MAIN_THREAD };
        self.register_thread(main_thread);
    }

    pub fn register_thread(&mut self, thread: &'static Thread<'static>) {
        self.tasks.push_front(&thread);
        self.count += 1;
    }

    pub fn print_tasks(&self) {
        println!("print_tasks (cur: {} count: {})", self.current, self.count);
        for task in self.tasks.iter() {
            println!("{}", task);
        }
    }

    pub fn current(&self) -> &'static Thread {
        for (index, task) in self.tasks.iter().enumerate() {
            if self.current == index {
                return task;
            }
        }
        panic!("Invalid current index");
    }

    pub fn current_ptr(&'static self) -> *mut Thread {
        let current = self.current();
        current as *const Thread as *mut Thread
    }

    // TODO: Remove the Cs parameter, access to Kernel is already atomic
    pub fn increment_ticks(&mut self, _cs: &Cs<GlobalIrq>) {
        self.ticks += 1;
    }

    // TODO: Remove the Cs parameter, access to Kernel is already atomic
    pub fn get_ticks(&self, _cs: &Cs<GlobalIrq>) -> u64 {
        self.ticks
    }

    // TODO: Remove the cs, access to Kernel is already atomic
    pub fn busy_wait(&self, ms: u32) {
        let end = atomic_restore(|cs| self.get_ticks(cs)).saturating_add(((ms * F) / 1000) as u64);
        while atomic_restore(|cs| self.get_ticks(cs)) < end {}
    }

    pub fn sched_next_thread(&mut self) {
        self.current = (self.current + 1) % self.count;
    }
}
