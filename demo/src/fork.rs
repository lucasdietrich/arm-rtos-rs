use core::ffi::c_void;

use crate::entry::USER_THREAD_SIZE;
use kernel::{
    kernel::{stack::Stack, thread::Thread, timeout::Timeout, userspace, CpuVariant},
    println,
};

pub fn init_threads<'a, CPU: CpuVariant>() -> [Thread<'a, CPU>; 1] {
    // initialize task1
    #[link_section = ".noinit"]
    static mut THREAD_STACK1: Stack<USER_THREAD_SIZE> = Stack::uninit();
    let stack1 = unsafe { THREAD_STACK1.get_info() };
    let task1 = Thread::init(&stack1, thread_fork, 0xaaaa0000 as *mut c_void, 0);

    [task1]
}

extern "C" fn thread_fork(_arg: *mut c_void) -> ! {
    let res = userspace::k_fork();
    println!("fork: res = {}", res);

    userspace::k_stop();
}
