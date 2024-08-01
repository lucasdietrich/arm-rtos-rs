use core::{ffi::c_void, ptr::null_mut};

use crate::{
    kernel::sleep,
    threading::{Stack, Thread},
};

extern "C" fn task_entry(arg: *mut c_void) -> ! {
    loop {
        sleep(1000);
    }

    loop {}
}

#[link_section = ".noinit"]
static mut TASK_STACK: [u8; 1024] = [0; 1024];

pub fn start_task() {
    let stack = Stack::new(unsafe { &mut TASK_STACK });
    Thread::init(&stack, task_entry, null_mut());
}
