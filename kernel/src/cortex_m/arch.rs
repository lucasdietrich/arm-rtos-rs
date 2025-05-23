use core::arch::{asm, global_asm};

use crate::kernel::{
    elf_loader::{Lex, PICRegImpl},
    CpuVariant, ExceptionStackFrame, ThreadEntry,
};

// Stack frame produced by an exception
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct __exception_sf {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r12: u32,
    pub lr: u32, // r14 (unset on thread entry)
    pub pc: u32, // r15 (return address ra in some context)
    pub xpsr: u32,
}

impl ExceptionStackFrame for __exception_sf {
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
            (*sf).pc = entry as usize as u32; // return address: task entry function address
            (*sf).xpsr = XPSR;
        };
    }
}

// Representation of the callee saved context in stack
// WARNING: This structure is not 8B aligned !
// This might be moved to thread structure to avoid SP aligned issues
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub struct __callee_context {
    pub v1: u32, // r4
    pub v2: u32, // r5
    pub v3: u32, // r6
    pub v4: u32, // r7
    pub v5: u32, // r8
    pub v6: u32, // r9
    pub v7: u32, // r10
    pub v8: u32, // r11
    pub ip: u32, // r12
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
    const FCPU: u32 = 25_000_000;

    // Add types for interrupts handlers like sytick, pendsv, svc

    type CalleeContext = __callee_context;
    type InitStackFrame = __exception_sf;

    #[cfg(any(
        not(any(feature = "loadable-elf-reg-r9", feature = "loadable-elf-reg-r10")),
        all(feature = "loadable-elf-reg-r9", feature = "loadable-elf-reg-r10")
    ))]
    compile_error!("One and only one PIC register must be selected");

    #[cfg(feature = "loadable-elf-reg-r9")]
    type PICRegImpl = R9;
    #[cfg(feature = "loadable-elf-reg-r10")]
    type PICRegImpl = R10;

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
            ldmia r1, {{r4-r11}}
    
            // 4. trigger a pendSV: set PENDSVSET bit (28) in ICSR register (0xE000ED04)
            ldr r0, =0xE000ED04   // Load ICSR address
            ldr r3, =0x10000000   // Load PENDSVSET bit value
            str r3, [r0]          // Trigger PendSV by writing to ICSR
            isb
    
            // =============================================================
            // PendSV triggered; now we have returned from the exception 
            // after a PendSV called by the user process
            // =============================================================
    
            // 5. Save user process context
            stmia r1, {{r4-r11}}
    
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
    .extern Z_SYSCALL_FLAG
    .thumb_func
z_svc:
    // SVC manages syscall and switch to the kernel

    // 1. Switch to priviledged mode
    mov r0, #0
    msr CONTROL, r0

    // 2. sync barrier required after CONTROL, from armv7 manual:
    // 'Software must use an ISB barrier instruction to ensure
    //  a write to the CONTROL register takes effect before the
    //  next instruction is executed.'
    isb

    // 3. Write 1 to Z_SYSCALL_FLAG variable
    ldr r0, =Z_SYSCALL_FLAG
    mov r1, #1
    str r1, [r0]

    // 4. load EXC_RETURN value to return in supervisor stack
    ldr lr, =0xFFFFFFF9

    // 5. switch to kernel
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
    // Systick interrupt is executed with the highest priority and 
    // cannot be preempted This is a *natural* critical section with 
    // the maximum degree

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

/// Implementation of `PICRegImpl` using register `r9` for the Global Offset Table.
#[derive(Debug)]
pub struct R9;

impl PICRegImpl for R9 {
    unsafe fn invoke_loadable_entry(lex: &Lex) -> u32 {
        let r0: u32;
        asm!(
            "
            blx {entry}
            ",
            entry = in(reg) lex.entry,
            inout("r0") lex.arg0 => r0,
            in("r9") lex.got_addr,
        );
        r0
    }
}

/// Implementation of `PICRegImpl` using register `r10` for the Global Offset Table.
#[derive(Debug)]
pub struct R10;

impl PICRegImpl for R10 {
    unsafe fn invoke_loadable_entry(lex: &Lex) -> u32 {
        let r0: u32;
        asm!(
            "
            blx {entry}
            ",
            entry = in(reg) lex.entry,
            inout("r0") lex.arg0 => r0,
            in("r10") lex.got_addr,
        );
        r0
    }
}
