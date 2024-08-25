use core::{
    arch::{asm, global_asm},
    ffi::c_void,
    mem::{self, MaybeUninit},
    task,
};

use crate::{
    io::{self, write_bytes},
    mps2_an385::UART0,
    println,
    serial_utils::Hex,
    threading::Thread,
};

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
        self.ticks
    }

    pub fn busy_wait(&self, ms: u32) {
        let end = self.get_ticks().saturating_add(((ms * F) / 1000) as u64);
        while self.get_ticks() < end {}
    }
}

#[repr(C)]
struct SVCCallParams {
    pub r0: *mut c_void,
    pub r1: *mut c_void,
    pub r2: *mut c_void,
    pub r3: *mut c_void,
    pub syscall_id: u32,
}

#[no_mangle]
extern "C" fn do_syscall(params: *const SVCCallParams) {
    let params = unsafe { &*params };

    match params.syscall_id {
        1 => {
            println!("Sleeping...");
        }
        2 => {
            let ptr = params.r0 as *const u8;
            let len = params.r1 as usize;

            // rebuild &[u8] from (string and len)
            let slice = unsafe { core::slice::from_raw_parts(ptr, len) };

            // Direct write
            io::write_bytes(slice);
        }
        0xbadf00d => {
            println!("BEEF!");
        }
        _ => {
            println!("Unknown syscall: {}", params.syscall_id);
        }
    }
}

global_asm!(
    "
    .section .text, \"ax\"
    .global z_svc
    .global do_syscall
    .thumb_func
z_svc:
    push {{r4, lr}}

    // Allocate space on the stack for SVCCallParams
    sub sp, sp, #20         // Allocate 20 bytes (5 * 4 bytes for r0, r1, r2, r3, syscall_id)

    // Store r0-r3 in the allocated stack space
    str r0, [sp, #0]        // params.r0 = r0
    str r1, [sp, #4]        // params.r1 = r1
    str r2, [sp, #8]        // params.r2 = r2
    str r3, [sp, #12]       // params.r3 = r3

    // Store r4 (syscall ID) in the allocated stack space
    str r4, [sp, #16]       // params.syscall_id = r4

    // Pass the pointer to params (sp) as an argument to do_syscall
    mov r0, sp              // r0 = params (stack pointer)

    // Call do_syscall function
    bl do_syscall

    // Clean up the stack
    add sp, sp, #20         // Deallocate the 20 bytes of stack space

    // Return from the function
    pop {{r4, pc}}
    "
);

extern "C" {
    pub fn z_svc();
}
