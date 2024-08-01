use core::{
    ffi::c_void,
    fmt::Display,
    ptr::{addr_of, null_mut},
};

// This function can be naked as it will never return !
type ThreadEntry = extern "C" fn(*mut c_void) -> !;

pub struct Stack {
    pub stack_end: *mut u32,
    pub stack_size: usize,
}

impl Stack {
    pub fn new(stack: &'static mut [u8]) -> Self {
        let stack_size = stack.len();
        Stack {
            stack_end: unsafe { stack.as_mut_ptr().add(stack_size) } as *mut u32,
            stack_size,
        }
    }
}

#[repr(C)]
pub struct Thread {
    pub stack_ptr: *mut u32,
}

impl Thread {
    pub const fn uninit() -> Self {
        Thread {
            stack_ptr: null_mut(),
        }
    }

    pub fn is_initialized(&self) -> bool {
        !self.stack_ptr.is_null()
    }

    pub fn init(stack: &Stack, entry: ThreadEntry, arg1: *mut c_void) -> Self {
        const UNDEFINED_MARKER: u32 = 0xAAAAAAAA;
        const XPSR: u32 = 0x41000000;

        // Stack must be initilized as follow (top to bottom)

        let init_stack_frame = [
            // LOW STACK ADDR
            // thread switch context
            // r4
            UNDEFINED_MARKER,
            // r5
            UNDEFINED_MARKER,
            // r6
            UNDEFINED_MARKER,
            // r7
            UNDEFINED_MARKER,
            // r8
            UNDEFINED_MARKER,
            // r9
            UNDEFINED_MARKER,
            // r10
            UNDEFINED_MARKER,
            // r11
            UNDEFINED_MARKER,
            // lr: exception
            0xFFFFFF9, // TODO
            // PENDSV context
            // r0: arg
            arg1 as u32,
            // r1: 0
            UNDEFINED_MARKER,
            // r2: 0
            UNDEFINED_MARKER,
            // r3: 0
            UNDEFINED_MARKER,
            // r12: 0
            UNDEFINED_MARKER,
            // lr: return address of the task entry function (TODO what value should be set ?)
            UNDEFINED_MARKER,
            // return address: task entry function address
            entry as u32,
            // END OF THREAD STACK
            // return program status register (xPSR)
            XPSR,
        ];

        let thread = Thread {
            stack_ptr: unsafe { stack.stack_end.sub(init_stack_frame.len()) },
        };

        for (index, val) in init_stack_frame.into_iter().enumerate() {
            unsafe { thread.stack_ptr.add(index).write(val) };
        }

        thread
    }
}

impl Display for Thread {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Thread sp=0x{:08x}", self.stack_ptr as u32)
    }
}
