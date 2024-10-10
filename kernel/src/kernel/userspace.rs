use core::{arch::asm, ffi::c_void, fmt::Arguments, ptr};

use super::syscalls::{IoSyscallId, KernelSyscallId, SyscallId};

// Compiler update should do the job:
//
// Generic const is asm requires "#![feature(asm_const)]"
pub unsafe fn z_call_svc_4<const SVC_NUM: u8>(mut r0: u32, r1: u32, r2: u32, r3: u32) -> i32 {
    asm!(
        "svc #{svc_num}",
        svc_num = const SVC_NUM,
        inlateout("r0") r0,
        in("r1") r1,
        in("r2") r2,
        in("r3") r3,
        options(nostack, nomem),
    );
    r0 as i32
}

pub fn k_svc_yield() -> i32 {
    unsafe { z_call_svc_4::<{ SyscallId::Kernel as u8 }>(0, 0, 0, KernelSyscallId::Yield as u32) }
}

pub fn k_svc_sleep(ms: u32) -> i32 {
    unsafe { z_call_svc_4::<{ SyscallId::Kernel as u8 }>(ms, 0, 0, KernelSyscallId::Sleep as u32) }
}

pub fn k_svc_print(string: &str) -> i32 {
    unsafe {
        z_call_svc_4::<{ SyscallId::Io as u8 }>(
            string.as_ptr() as u32,
            string.len() as u32,
            0,
            IoSyscallId::Print as u32,
        )
    }
}

// pub fn z_user_print(args: Arguments<'_>, nl: bool) {

// }

// #[macro_export]
// macro_rules! z_user_print {
//     () => {};
//     ($($arg:tt)*) => {{
//         $crate::io::_print(format_args!($($arg)*), false)
//     }};
// }

// #[macro_export]
// macro_rules! user_println {
//     () => {
//         $crate::io::_print(format_args!("\n"), false)
//     };
//     ($($arg:tt)*) => {{
//         $crate::io::_print(format_args!($($arg)*), true)
//     }}
// }
