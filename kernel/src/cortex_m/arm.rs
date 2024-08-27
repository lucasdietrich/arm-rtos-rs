use super::systick::SysTick;

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

// Representation of the callee saved context in stack
// WARNING: This structure is not 8B aligned !
// This might be moved to thread structure to avoid SP aligned issues
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct __callee_context {
    pub v1: u32,
    pub v2: u32,
    pub v3: u32,
    pub v4: u32,
    pub v5: u32,
    pub v6: u32,
    pub v7: u32,
    pub v8: u32,
    pub ip: u32,
}

pub const SCS_BASE: usize = 0xE000E000;

pub const SYSTICK_BASE: usize = SCS_BASE + 0x0010;
pub const SYSTICK: *mut SysTick = SYSTICK_BASE as *mut SysTick;
