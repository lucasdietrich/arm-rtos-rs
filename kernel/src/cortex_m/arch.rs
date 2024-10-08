use core::arch::{asm, global_asm};

use crate::kernel::{CpuVariant, InitStackFrameTrait, ThreadEntry};

// Stack frame produced by an exception
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct __basic_sf {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r12: u32,
    pub lr: u32, // r14 (unset on thread entry)
    pub pc: u32, // r15 (return address ra in some context)
    pub xpsr: u32,
}

impl InitStackFrameTrait for __basic_sf {
    fn initialize_at(stack_ptr: *mut u32, entry: ThreadEntry, arg0: *mut core::ffi::c_void) {
        // TODO: Change this value to something not significant (e.g. 0x00000000)
        const UNDEFINED_MARKER: u32 = 0xAAAAAAAA;
        // TODO: Change this value to something not significant (e.g. 0x00000000)
        const LR_DEFAULT: u32 = 0xFFFFFFFF;

        // Thumb bit to 1
        const XPSR: u32 = 0x01000000;

        let sf = stack_ptr as *mut Self;

        // Create exception stack frame
        unsafe {
            (*sf).r0 = arg0 as u32;
            (*sf).r1 = UNDEFINED_MARKER;
            (*sf).r2 = UNDEFINED_MARKER;
            (*sf).r3 = UNDEFINED_MARKER;
            (*sf).r12 = UNDEFINED_MARKER;
            (*sf).lr = LR_DEFAULT;
            (*sf).pc = entry as u32; // return address: task entry function address
            (*sf).xpsr = XPSR;
        };
    }
}

// Representation of the callee saved context in stack
// WARNING: This structure is not 8B aligned !
// This might be moved to thread structure to avoid SP aligned issues
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub struct __callee_context {
    pub v1: u32, // r4
    pub v2: u32,
    pub v3: u32,
    pub v4: u32,
    pub v5: u32,
    pub v6: u32,
    pub v7: u32,
    pub v8: u32,
    pub ip: u32,
}

impl __callee_context {
    pub const fn zeroes() -> Self {
        __callee_context {
            v1: 0,
            v2: 0,
            v3: 0,
            v4: 0,
            v5: 0,
            v6: 0,
            v7: 0,
            v8: 0,
            ip: 0,
        }
    }
}

impl Default for __callee_context {
    fn default() -> Self {
        Self::zeroes()
    }
}

pub struct CortexM;

impl CpuVariant for CortexM {
    type CalleeContext = __callee_context;
    type InitStackFrame = __basic_sf;

    #[export_name = "switch_to_user"]
    unsafe fn switch_to_user(
        mut stack_ptr: *mut u32,
        process_regs: *mut Self::CalleeContext,
    ) -> *mut u32 {
        asm!(
            "
            // 1. Save kernel call-saved registers on the stack
            push {{v1-v8, ip}}
    
            // 2. Set user stack pointer
            msr psp, r0
    
            // 3. Restore user process context
            ldmia r1, {{r4, r11}}
    
            // 4. trigger a pendSV: set PENDSVSET bit (28) in ICSR register (0xE000ED04)
            // 4.a)
            // ldr r0, =0xE000ED04
            // ldr r2, [r0, #0]
            // ldr r3, =0x10000000
            // orr r3, r3, r2
            // str r3, [r0]
            // isb
                
            // 4.b)
            ldr r0, =0xE000ED04   // Load ICSR address
            ldr r3, =0x10000000   // Load PENDSVSET bit value
            str r3, [r0]          // Trigger PendSV by writing to ICSR
            isb
    
            // 4.c)
            // svc 0xFF
    
            // =============================================================
            // PendSV triggered; now we have returned from the exception 
            // after a PendSV called by the user process
            // =============================================================
    
            // 5. Save user process context
            stmia r1, {{r4, r11}}
    
            // 6. Save user process stack pointer back to r0
            mrs r0, psp
    
            // 7. Pop kernel call-saved registers from the stack
            pop {{v1-v8, ip}}
        
            ",
            inout("r0") stack_ptr,
            in("r1") process_regs,
        );

        stack_ptr
    }
}

// 1. Calls to pendsv saves:
//  r0-r3, r12, lr, return addr, xpsr
global_asm!(
    "
    .section .text, \"ax\"
    .global z_svc
    .thumb_func
z_svc:
    // SVC manages switch to the kernel

    // 1. Switch to priviledged mode
    mov r0, #0
    msr CONTROL, r0

    // 2. sync barrier required after CONTROL, from armv7 manual:
    // 'Software must use an ISB barrier instruction to ensure
    //  a write to the CONTROL register takes effect before the
    //  next instruction is executed.'
    isb

    // 3. load EXC_RETURN value to return in supervisor stack
    ldr lr, =0xFFFFFFF9

    // 4. switch to kernel
    bx lr
    "
);

// 1. Calls to pendsv saves:
//  r0-r3, r12, lr, return addr, xpsr
global_asm!(
    "
    .section .text, \"ax\"
    .global z_pendsv
    .thumb_func
z_pendsv:
    // PendSV manages final changes to switch to the user process

    // 1. Switch to unpriviledged mode
    mov r0, #1
    msr CONTROL, r0

    // 2. sync barrier required after CONTROL, from armv7 manual:
    // 'Software must use an ISB barrier instruction to ensure 
    //  a write to the CONTROL register takes effect before the 
    //  next instruction is executed.'
    isb

    // 3. load EXC_RETURN value to return in process stack
    ldr lr, =0xFFFFFFFD

    // 4. switch to user
    bx lr
    "
);

global_asm!(
    "
    .section .text, \"ax\"
    .global z_systick
    .thumb_func
z_systick:
    // // Systick interrupt is executed with the highest priority and 
    // // cannot be preempted This is a *natural* critical section with 
    // // the maximum degree

    // // 1. Switch to priviledged mode
    // mov r0, #0
    // msr CONTROL, r0

    // // 2. sync barrier required after CONTROL, from armv7 manual:
    // // 'Software must use an ISB barrier instruction to ensure
    // //  a write to the CONTROL register takes effect before the
    // //  next instruction is executed.'
    // isb

    // // 3. load EXC_RETURN value to return in supervisor stack
    // ldr lr, =0xFFFFFFF9

    // 4. switch to kernel
    bx lr
    "
);
