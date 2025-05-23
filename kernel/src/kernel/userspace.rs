use core::{
    arch::asm,
    fmt::{Arguments, Write},
};

use super::{
    syscalls::{IoSyscallId, KernelSyscallId, SyncPrimitiveType, SyscallId},
    timeout::Timeout,
};

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
    unsafe { z_call_svc_kernel_4(0, 0, 0, KernelSyscallId::Yield as u32) }
}

pub fn k_sleep(duration: Timeout) -> i32 {
    let r0: i32 = duration.into();
    unsafe { z_call_svc_kernel_4(r0 as u32, 0, 0, KernelSyscallId::Sleep as u32) }
}

pub fn k_print(string: &str, nl: bool) -> i32 {
    unsafe {
        z_call_svc_4::<{ SyscallId::Io as u8 }>(
            string.as_ptr() as u32,
            string.len() as u32,
            nl as u32,
            IoSyscallId::Write as u32,
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

pub fn k_mutex_lock(mutex: i32, timeout: Timeout) -> i32 {
    let r0: i32 = timeout.into();
    unsafe {
        z_call_svc_kernel_4(
            r0 as u32,
            mutex as u32,
            SyncPrimitiveType::Mutex as u32,
            KernelSyscallId::Pend as u32,
        )
    }
}

pub fn k_mutex_unlock(mutex: i32) -> i32 {
    unsafe {
        z_call_svc_kernel_4(
            0,
            mutex as u32,
            SyncPrimitiveType::Mutex as u32,
            KernelSyscallId::Sync as u32,
        )
    }
}

pub fn k_sync(kobj: i32) -> i32 {
    unsafe {
        z_call_svc_kernel_4(
            0,
            kobj as u32,
            SyncPrimitiveType::Sync as u32,
            KernelSyscallId::Sync as u32,
        )
    }
}

pub fn k_signal(kobj: i32, signal_value: u32) -> i32 {
    unsafe {
        z_call_svc_kernel_4(
            signal_value,
            kobj as u32,
            SyncPrimitiveType::Signal as u32,
            KernelSyscallId::Sync as u32,
        )
    }
}

pub fn k_signal_poll(kobj: i32, timeout: Timeout) -> i32 {
    let r0: i32 = timeout.into();
    unsafe {
        z_call_svc_kernel_4(
            r0 as u32,
            kobj as u32,
            SyncPrimitiveType::Signal as u32,
            KernelSyscallId::Pend as u32,
        )
    }
}

pub fn k_pend(kobj: i32, timeout: Timeout) -> i32 {
    let r0: i32 = timeout.into();
    unsafe {
        z_call_svc_kernel_4(
            r0 as u32,
            kobj as u32,
            SyncPrimitiveType::Sync as u32,
            KernelSyscallId::Pend as u32,
        )
    }
}

pub fn k_fork() -> i32 {
    unsafe { z_call_svc_kernel_4(0, 0, 0, KernelSyscallId::Fork as u32) }
}

pub fn k_stop() -> ! {
    let _ = unsafe { z_call_svc_kernel_4(0, 0, 0, KernelSyscallId::Stop as u32) };
    unreachable!()
}

pub fn k_stdio_read1() -> Option<u8> {
    let ret =
        unsafe { z_call_svc_4::<{ SyscallId::Io as u8 }>(0, 0, 0, IoSyscallId::Read1 as u32) };
    if ret < 0 {
        None
    } else {
        Some(ret as u8)
    }
}

pub fn z_user_print(args: Arguments<'_>, nl: bool) {
    struct UserIo(bool); // Boolean parameter to print a newline

    impl Write for UserIo {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let ret = k_print(s, self.0);
            if ret < 0 {
                Err(core::fmt::Error)
            } else {
                Ok(())
            }
        }
    }

    let _ = UserIo(nl).write_fmt(args);
}

#[macro_export]
macro_rules! user_print {
    () => {};
    ($($arg:tt)*) => {{
        $crate::kernel::userspace::z_user_print(format_args!($($arg)*), false)
    }};
}

#[macro_export]
macro_rules! user_println {
    () => {
        $crate::kernel::userspace::z_user_print(format_args!("\n"), false)
    };
    ($($arg:tt)*) => {{
        $crate::kernel::userspace::z_user_print(format_args!($($arg)*), true)
    }}
}
