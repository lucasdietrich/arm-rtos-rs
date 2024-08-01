use core::{
    arch::{asm, global_asm},
    task,
};

use crate::{println, threading::Thread};

pub fn sleep(ms: u32) {}

// 1. Calls to pendsv saves:
//  r0-r3, r12, lr, return addr, xpsr
// 2. 'from' thread's SP addr is written to r0
// 3. 'to' thread's SP addr is written to r1
// r4-r11 must be saved
global_asm!(
    "
    .section .text, \"ax\"
    .global _thread_switch
    .thumb_func
_thread_switch:
_thread_switch_push:
    # save 'from' thread context
    push {{r4-r11, lr}}

    # save sp to 'from' thread
    str sp, [r0]

_thread_switch_pop:
    # load sp from 'from' thread
    ldr sp, [r1]

    # restore 'to' thread context
    pop {{r4-r11, pc}}
    "
);

extern "C" {
    pub fn _thread_switch(from: *mut Thread, to: *const Thread);
}

pub struct Kernel<const N: usize = 1> {
    tasks: [Option<Thread>; N],
    current: usize,
}

impl<const N: usize> Kernel<N> {
    pub const fn init() -> Kernel<N> {
        // Create an uninitialized array of MaybeUninit
        let mut tasks = [const { None }; N];

        tasks[0] = Some(Thread::uninit());

        Kernel { tasks, current: 0 }
    }

    pub fn register_thread(&mut self, thread: Thread) -> Result<(), Thread> {
        if let Some(slot) = self.tasks.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(thread);
            Ok(())
        } else {
            Err(thread)
        }
    }

    pub fn pendsv_handler(&mut self) {
        self.yield_next();
    }

    fn yield_current(&mut self) {
        let current = self.current();

        println!("{}", current);

        unsafe { _thread_switch(current as *mut Thread, current as *const Thread) }
    }

    fn yield_next(&mut self) {
        let index_next = (self.current + 1) % N;
        let next = self.tasks[index_next].as_mut().unwrap() as *const Thread; // Remove the unwrap
        let current = self.current() as *mut Thread;

        unsafe { _thread_switch(current as *mut Thread, next as *const Thread) }
    }

    pub fn current(&mut self) -> &mut Thread {
        self.tasks[self.current].as_mut().unwrap()
    }
}
