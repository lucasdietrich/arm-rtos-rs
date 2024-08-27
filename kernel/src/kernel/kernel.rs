use core::arch::global_asm;

use crate::cortex_m::{
    critical_section::Cs,
    interrupts::{atomic_restore, atomic_section},
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
pub struct Kernel<const N: usize = 1, const F: u32 = 1> {
    tasks: [Option<Thread>; N],
    current: usize,

    // Ticks counter: period: P (ms)
    ticks: u64,
}

impl<const N: usize, const F: u32> Kernel<N, F> {
    pub const fn init() -> Kernel<N, F> {
        // Create an uninitialized array of MaybeUninit
        let mut tasks = [const { None }; N];

        tasks[0] = Some(Thread::uninit());

        Kernel {
            tasks,
            current: 0,
            ticks: 0,
        }
    }

    pub fn start(&mut self, _cs: Cs<Kernel>) {}

    pub fn register_thread(&mut self, thread: Thread) -> Result<*mut Thread, Thread> {
        if let Some(slot) = self.tasks.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(thread);
            Ok(slot.as_mut().unwrap() as *mut Thread)
        } else {
            Err(thread)
        }
    }

    // fn yield_current(&mut self) {
    //     let current = self.current();

    //     unsafe { _thread_switch(current as *mut Thread, current as *const Thread) }
    // }

    // fn yield_next(&mut self) {
    //     let index_next = (self.current + 1) % N;
    //     let next = self.tasks[index_next].as_mut().unwrap() as *const Thread; // Remove the unwrap
    //     let current = self.current() as *mut Thread;

    //     unsafe { _thread_switch(current as *mut Thread, next as *const Thread) }
    // }

    pub fn current(&mut self) -> &mut Thread {
        self.tasks[self.current].as_mut().unwrap()
    }

    pub unsafe fn get_current_ptr(&mut self) -> *mut Thread {
        self.tasks[self.current].as_mut().unwrap() as *mut Thread
    }

    // TODO Any race condition on the ticks counter ?
    pub fn increment_ticks(&mut self) {
        self.ticks += 1;
    }

    // TODO Any race condition on the ticks counter ?
    pub fn get_ticks(&self) -> u64 {
        atomic_restore(|_cs| self.ticks)
    }

    pub fn busy_wait(&self, ms: u32) {
        let end = self.get_ticks().saturating_add(((ms * F) / 1000) as u64);
        while self.get_ticks() < end {}
    }
}
