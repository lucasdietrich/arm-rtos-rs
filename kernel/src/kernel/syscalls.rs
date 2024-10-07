use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use core::arch::global_asm;

use crate::{
    cortex_m::critical_section::Cs,
    kernel::errno::Kerr,
    println,
    stdio::{self},
};

use super::kernel::Kernel;

#[repr(u32)]
#[derive(FromPrimitive)]
pub enum SyscallId {
    SLEEP = 1,
    PRINT = 2,
    YIELD = 3,
}

#[repr(C)]
struct SVCCallParams {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub syscall_id: u32,
}

fn sys_sleep(duration: u32) -> i32 {
    println!("Sleeping...");
    Kerr::ENOSYS as i32
}

#[no_mangle]
extern "C" fn do_syscall(params: *const SVCCallParams) -> i32 {
    let cs = unsafe { Cs::<Kernel>::new() };

    let params = unsafe { &*params };

    if let Some(syscall_id) = FromPrimitive::from_u32(params.syscall_id) {
        match syscall_id {
            SyscallId::SLEEP => sys_sleep(params.r0),
            SyscallId::PRINT => {
                let ptr = params.r0 as *const u8;
                let len = params.r1 as usize;

                // rebuild &[u8] from (string and len)
                let slice = unsafe { core::slice::from_raw_parts(ptr, len) };

                // Direct write
                stdio::write_bytes(slice);
                0
            }
            SyscallId::YIELD => {
                println!("Yield...");
                0
            }
        }
    } else {
        println!("Unknown syscall: {}", params.syscall_id);
        0
    }
}

// global_asm!(
//     "
//     .section .text, \"ax\"
//     .global z_svc
//     .global do_syscall
//     .thumb_func
// z_svc:
//     // At this point, the exception frame looks like this
//     // sp + 00: r0 (syscall arg 0)
//     // sp + 04: r1 (syscall arg 1)
//     // sp + 08: r2 (syscall arg 2)
//     // sp + 0C: r3 (syscall arg 3)
//     // sp + 10: r12
//     // sp + 14: lr
//     // sp + 18: return address (instruction following the svc)
//     // sp + 1C: xPSR

//     push {{r4, lr}}

//     // Allocate space on the stack for SVCCallParams
//     sub sp, sp, #20         // Allocate 20 bytes (5 * 4 bytes for r0, r1, r2, r3, syscall_id)

//     // Store r0-r3 in the allocated stack space
//     str r0, [sp, #0]        // params.r0 = r0
//     str r1, [sp, #4]        // params.r1 = r1
//     str r2, [sp, #8]        // params.r2 = r2
//     str r3, [sp, #12]       // params.r3 = r3

//     // Store r4 (syscall ID) in the allocated stack space
//     str r4, [sp, #16]       // params.syscall_id = r4

//     // Pass the pointer to params (sp) as an argument to do_syscall
//     mov r0, sp              // r0 = params (stack pointer)

//     // Call do_syscall function
//     bl do_syscall

//     // r0 contains do_syscall returned value

//     // Clean up the stack
//     add sp, sp, #20         // Deallocate the 20 bytes of stack space

//     // Replace value of r0 in the exception stack frame, so that when the
//     // exception returns. The return value of the syscall is automatically
//     // set in r0
//     str r0, [sp, #8]

//     // At this point, the exception frame looks like this
//     // sp + 00: next pc (old lr)
//     // sp + 04: next r4 (old r4)
//     // sp + 08: SYSCALL RETURN VALUE (old r0)
//     // sp + 0C: r1 (syscall arg 1)
//     // sp + 10: r2 (syscall arg 2)
//     // sp + 14: r3 (syscall arg 3)
//     // sp + 18: r12
//     // sp + 1C: lr
//     // sp + 20: return address (instruction following the svc)
//     // sp + 24: xPSR

//     // Return from the function
//     pop {{r4, pc}}
//     "
// );

// extern "C" {
//     pub fn z_svc();
// }
