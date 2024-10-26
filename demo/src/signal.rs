use core::ffi::c_void;

use crate::entry::USER_THREAD_SIZE;
use kernel::{
    kernel::{stack::Stack, thread::Thread, timeout::Timeout, userspace, CpuVariant},
    println,
};

pub fn init_threads<'a, CPU: CpuVariant>() -> [Thread<'a, CPU>; 3] {
    // initialize task1
    #[link_section = ".noinit"]
    static mut THREAD_STACK1: Stack<USER_THREAD_SIZE> = Stack::uninit();
    let stack1 = unsafe { THREAD_STACK1.get_info() };
    let task1 = Thread::init(&stack1, signal_consumer, 0xaaaa0000 as *mut c_void, 0);

    // initialize task2
    #[link_section = ".noinit"]
    static mut THREAD_STACK2: Stack<USER_THREAD_SIZE> = Stack::uninit();
    let stack2 = unsafe { THREAD_STACK2.get_info() };
    let task2 = Thread::init(&stack2, signal_consumer, 0xbbbb0000 as *mut c_void, 0);

    // initialize task3
    #[link_section = ".noinit"]
    static mut THREAD_STACK3: Stack<USER_THREAD_SIZE> = Stack::uninit();
    let stack3 = unsafe { THREAD_STACK3.get_info() };
    let task3 = Thread::init(&stack3, signal_producer, 0xcccc0000 as *mut c_void, 0);

    [task1, task2, task3]
}

extern "C" fn signal_consumer(_arg: *mut c_void) -> ! {
    userspace::k_sleep(Timeout::from_ms(1000));
    loop {
        let signal_val = userspace::k_signal_poll(0, Timeout::from_ms(3000));
        println!("consumer: poll signal = {}", signal_val);

        if signal_val >= 0 {
            break;
        }
    }

    println!("consumer: done");
    userspace::k_stop();
}

extern "C" fn signal_producer(_arg: *mut c_void) -> ! {
    let signal = userspace::k_signal_create();
    println!("producer: create signal = {}", signal);

    let signal_value = 12345;

    userspace::k_sleep(Timeout::from_ms(5000));
    let ret = userspace::k_signal(signal, signal_value);
    println!("producer: signal = {}, ret = {}", signal, ret);

    println!("producer: done");
    userspace::k_stop();
}
