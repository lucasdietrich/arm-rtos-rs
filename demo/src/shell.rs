use core::ffi::c_void;

use crate::entry::USER_THREAD_SIZE;
use kernel::{
    kernel::{stack::Stack, thread::Thread, timeout::Timeout, userspace, CpuVariant},
    println,
    serial_utils::Hex,
    stdio,
};

pub fn init_shell_thread<'a, CPU: CpuVariant>() -> Thread<'a, CPU> {
    #[link_section = ".noinit"]
    static mut THREAD_STACK_SHELL: Stack<USER_THREAD_SIZE> = Stack::uninit();
    let stack = unsafe { THREAD_STACK_SHELL.get_info() };
    let thread = Thread::init(&stack, mytask_shell, 0xaaaa0000 as *mut c_void, 0);

    thread
}

extern "C" fn mytask_shell(_arg: *mut c_void) -> ! {
    loop {
        if let Some(byte) = userspace::k_stdio_read1() {
            println!("recv: {}", Hex::U8(byte));

            let mut syscall_ret = 0;

            match byte {
                b'y' => {
                    println!("yield !");
                    userspace::k_yield();
                }
                b's' => {
                    println!("SVC sleep");
                    syscall_ret = userspace::k_sleep(Timeout::from_ms(1000));
                }
                b'w' => {
                    println!("SVC print");
                    let msg = "Hello using SVC !!\n";
                    syscall_ret = userspace::k_print(msg);
                }
                _ => {}
            }

            println!("syscall_ret: {}", Hex::U32(syscall_ret as u32));
        }

        userspace::k_sleep(Timeout::from_ms(100));
    }
}
