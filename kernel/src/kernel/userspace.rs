use core::arch::asm;

use super::syscalls::{IoSyscallId, KernelSyscallId, SyncPrimitiveType, SyscallId};

// Compiler update should do the job:
//
// Generic const in asm requires "#![feature(asm_const)]"
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

pub unsafe fn z_call_svc_kernel_4(r0: u32, r1: u32, r2: u32, r3: u32) -> i32 {
    z_call_svc_4::<{ SyscallId::Kernel as u8 }>(r0, r1, r2, r3)
}

pub fn k_yield() -> i32 {
    unsafe { z_call_svc_4::<{ SyscallId::Kernel as u8 }>(0, 0, 0, KernelSyscallId::Yield as u32) }
}

pub fn k_sleep(ms: u32) -> i32 {
    unsafe { z_call_svc_4::<{ SyscallId::Kernel as u8 }>(ms, 0, 0, KernelSyscallId::Sleep as u32) }
}

pub fn k_print(string: &str) -> i32 {
    unsafe {
        z_call_svc_4::<{ SyscallId::Io as u8 }>(
            string.as_ptr() as u32,
            string.len() as u32,
            0,
            IoSyscallId::Print as u32,
        )
    }
}

pub fn k_sync_create() -> i32 {
    unsafe {
        z_call_svc_kernel_4(
            0,
            0,
            SyncPrimitiveType::Sync as u32,
            KernelSyscallId::SyncCreate as u32,
        )
    }
}

pub fn k_signal_create() -> i32 {
    unsafe {
        z_call_svc_kernel_4(
            0,
            0,
            SyncPrimitiveType::Signal as u32,
            KernelSyscallId::SyncCreate as u32,
        )
    }
}

pub fn k_semaphore_create(init: u32, max: u32) -> i32 {
    unsafe {
        z_call_svc_kernel_4(
            init,
            max,
            SyncPrimitiveType::Semaphore as u32,
            KernelSyscallId::SyncCreate as u32,
        )
    }
}

pub fn k_mutex_create() -> i32 {
    unsafe {
        z_call_svc_kernel_4(
            0,
            0,
            SyncPrimitiveType::Mutex as u32,
            KernelSyscallId::SyncCreate as u32,
        )
    }
}

pub fn k_sync(kobj: i32) -> i32 {
    unsafe { z_call_svc_kernel_4(0, 0, kobj as u32, KernelSyscallId::Sync as u32) }
}

pub fn k_pend(kobj: i32) -> i32 {
    unsafe { z_call_svc_kernel_4(0, 0, kobj as u32, KernelSyscallId::Pend as u32) }
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
