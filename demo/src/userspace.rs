use core::{arch::asm, ffi::c_void, fmt::Arguments, ptr};

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
unsafe extern "C" fn z_call_svc_4(
    r0: *mut c_void,
    r1: *mut c_void,
    r2: *mut c_void,
    r3: *mut c_void,
    syscall_id: u32,
) {
    // TODO change this value "#0xbb"
    asm!(
        "
        svc #0xbb
    ",
    in("r0") r0,
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
}

pub fn k_svc_debug() {
    unsafe {
        z_call_svc_4(
            0xaaaaaaaa as *mut c_void,
            0xbbbbbbbb as *mut c_void,
            0xcccccccc as *mut c_void,
            0xdddddddd as *mut c_void,
            SyscallId::BEEF as u32,
        )
    }
}

pub fn k_svc_yield() {
    unsafe {
        z_call_svc_4(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            SyscallId::YIELD as u32,
        )
    }
}

pub fn k_svc_sleep(ms: u32) {
    unsafe {
        z_call_svc_4(
            ms as *mut c_void,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            SyscallId::SLEEP as u32,
        )
    }
}

pub fn k_svc_print(string: &str) {
    unsafe {
        z_call_svc_4(
            string.as_ptr() as *mut c_void,
            string.len() as *mut c_void,
            ptr::null_mut(),
            ptr::null_mut(),
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
