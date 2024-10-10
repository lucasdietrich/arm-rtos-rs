use core::{arch::arm::__wfi, ffi::c_void, ptr};

use crate::println;

use super::{stack::Stack, thread::Thread, userspace, CpuVariant};

pub const IDLE_STACK_SIZE: usize = 1024;

#[link_section = ".noinit"]
static mut IDLE_STACK: Stack<IDLE_STACK_SIZE> = Stack::init();

pub struct Idle;

impl Idle {
    extern "C" fn idle_entry(arg0: *mut c_void) -> ! {
        loop {
            // unsafe { __wfi() };

            // println!("[IDLE] interrupt");

            // userspace::k_svc_yield();
        }
    }

    pub fn init<'a, CPU: CpuVariant>() -> Thread<'a, CPU> {
        let stack_info = unsafe { &mut IDLE_STACK }.get_info();

        Thread::init(
            &stack_info,
            Self::idle_entry,
            ptr::null_mut() as *mut c_void,
            0,
        )
    }
}
