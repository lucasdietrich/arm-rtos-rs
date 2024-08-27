use core::{arch::asm, ffi::c_void, fmt::Arguments, ptr};

use num_derive::FromPrimitive;

#[repr(u32)]
#[derive(FromPrimitive)]
pub enum SyscallId {
    SLEEP = 1,
    PRINT = 2,
    YIELD = 3,
    BEEF = 0xbadf00d,
}

// Read A7.7.175 of DDI0403E_B_armv7m_arm.pdf
// TODO how to read back svc value 0xbb
// -> read pc-4
#[no_mangle]
unsafe extern "C" fn z_call_svc_4(mut r0: u32, r1: u32, r2: u32, r3: u32, syscall_id: u32) -> i32 {
    // TODO change this value "#0xbb"
    asm!(
        "
        svc #0xbb
    ",
    inout("r0") r0,
    in("r1") r1,
    in("r2") r2,
    in("r3") r3,

    /* LLVM internally uses r6 which cannot be used in inline assembly
     * For comparison: Zephyr (based on GCC/Clang) uses r6 for syscalls.
     */
    in("r4") syscall_id,

    // Indication:
    options(nostack, nomem),
    );

    r0 as i32
}

pub fn k_svc_debug() -> i32 {
    unsafe {
        z_call_svc_4(
            0xaaaaaaaa,
            0xbbbbbbbb,
            0xcccccccc,
            0xdddddddd,
            SyscallId::BEEF as u32,
        )
    }
}

pub fn k_svc_yield() -> i32 {
    unsafe { z_call_svc_4(0, 0, 0, 0, SyscallId::YIELD as u32) }
}

pub fn k_svc_sleep(ms: u32) -> i32 {
    unsafe { z_call_svc_4(ms, 0, 0, 0, SyscallId::SLEEP as u32) }
}

pub fn k_svc_print(string: &str) -> i32 {
    unsafe {
        z_call_svc_4(
            string.as_ptr() as u32,
            string.len() as u32,
            0,
            0,
            SyscallId::PRINT as u32,
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
