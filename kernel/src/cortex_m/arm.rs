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
