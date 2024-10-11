use core::{
    arch::arm::__wfi,
    ffi::c_void,
    ptr::{self, addr_of_mut},
};

use super::{stack::Stack, thread::Thread, CpuVariant};

pub const IDLE_STACK_SIZE: usize = 1024;

#[link_section = ".noinit"]
static mut IDLE_STACK: Stack<IDLE_STACK_SIZE> = Stack::uninit();

pub struct Idle;

impl Idle {
    extern "C" fn idle_entry(_arg0: *mut c_void) -> ! {
        loop {
            unsafe { __wfi() };
        }
    }

    pub fn init<'a, CPU: CpuVariant>() -> Thread<'a, CPU> {
        let stack_info = unsafe { &mut *addr_of_mut!(IDLE_STACK) }.get_info();

        Thread::init(&stack_info, Self::idle_entry, ptr::null_mut(), 0)
    }
}
