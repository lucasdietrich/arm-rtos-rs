use core::{
    arch::{asm, global_asm},
    task,
};

use crate::{println, serial_utils::Hex, threading::Thread};

pub fn sleep(ms: u32) {}

// 1. Calls to pendsv saves:
//  r0-r3, r12, lr, return addr, xpsr
// 2. 'from' thread's SP addr is written to r0
// 3. 'to' thread's SP addr is written to r1
// r4-r11 must be saved
// global_asm!(
//     "
//     .section .text, \"ax\"
//     .global _thread_switch
//     .thumb_func
// _thread_switch:
//     # save 'from' thread context
//     push {{r4-r11, lr}}

//     # save sp to 'from' thread
//     str sp, [r0]

//     # load sp from 'from' thread
//     ldr sp, [r1]

//     # restore 'to' thread context
//     pop {{r4-r11, pc}}
//     "
// );

// 1. Calls to pendsv saves:
//  r0-r3, r12, lr, return addr, xpsr

#[link_section = ".bss"]
#[used]
pub static mut BSS_MYVAR: u32 = 0;

#[used]
#[no_mangle]
pub static mut z_current: *mut Thread = core::ptr::null_mut();

#[used]
#[no_mangle]
pub static mut z_next: *mut Thread = core::ptr::null_mut();

global_asm!(
    "
    .section .text, \"ax\"
    .global z_pendsv
    .thumb_func
z_pendsv:

    // save current lr (exception return) to r2
    // mov r2, lr

    // retrieve address of current thread and save it in r0
    ldr r2, =z_current
    ldr r0, [r2]
    ldr r3, =z_next // use z_kernel offsets
    ldr r1, [r3]

_thread_switch:
    // call debug function
    // b z_debug

    // save 'from' thread context
    push {{v1-v8, ip}}

    // save sp to 'from' thread
    str sp, [r0]

    // load sp from 'from' thread
    ldr sp, [r1]

    // restore 'to' thread context
    pop {{v1-v8, ip}}

    // invert z_current and z_next threads
    // str r2, [r1]
    // str r3, [r0]

    // load KERNEL structure into r0
    bx lr
    "
);

extern "C" {
    pub fn z_pendsv();
}

#[repr(C)]
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
}
