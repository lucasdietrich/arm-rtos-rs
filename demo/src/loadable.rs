use kernel::{
    self,
    kernel::{kernel::Kernel, CpuVariant},
    println, user_print,
};

use core::ffi::c_void;

use crate::entry::USER_THREAD_SIZE;
use kernel::kernel::{stack::Stack, thread::Thread, timeout::Timeout, userspace};

pub fn init_misc<'a, CPU: CpuVariant>() -> Thread<'a, CPU> {
    #[link_section = ".noinit"]
    static mut THREAD_STACK_LOADALE: Stack<USER_THREAD_SIZE> = Stack::uninit();
    let stack = unsafe { THREAD_STACK_LOADALE.get_info() };
    let thread = Thread::init(&stack, mytask_misc, 0xaaaa0000 as *mut c_void, 0);

    thread
}

extern "C" fn mytask_misc(_arg: *mut c_void) -> ! {
    loop {
        user_print!(".");
        userspace::k_sleep(Timeout::from_seconds(1));
    }
}

pub fn init<'a, CPU: CpuVariant, const K: usize, const F: u32>(
    ker: &mut Kernel<'a, CPU, K, F>,
) -> Thread<'a, CPU> {
    let thread = init_misc();

    let elf_bytes = include_bytes!("../../samples/hello_world.elf");

    match ker.load_elf(elf_bytes) {
        Ok(_) => println!("elf 1 loaded"),
        Err(e) => println!("Error loading elf: {:?}", e),
    }

    match ker.load_elf(elf_bytes) {
        Ok(_) => println!("elf 2 loaded"),
        Err(e) => println!("Error loading elf: {:?}", e),
    }

    thread
}
