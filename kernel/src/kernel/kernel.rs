use core::{
    arch::{asm, global_asm},
    ffi::c_void,
    u64,
};

use crate::{
    cortex_m::{
        arm::__callee_context,
        cortex_m_rt::k_call_pendsv,
        critical_section::{Cs, GlobalIrq},
        interrupts::{self, atomic_restore, atomic_section},
    },
    list, println,
    serial_utils::Hex,
    stdio,
};

use super::thread::Thread;

pub fn sleep(ms: u32) {}

#[link_section = ".bss"]
#[used]
pub static mut BSS_MYVAR: u32 = 0;

#[used]
#[no_mangle]
pub static mut z_current: *mut Thread = core::ptr::null_mut();

#[used]
#[no_mangle]
pub static mut z_next: *mut Thread = core::ptr::null_mut();

global_asm!(
    "
    .section .text, \"ax\"
    .global z_svc
    .thumb_func
z_svc:
    // SVC manages final changes to switch to the user process

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

#[export_name = "switch_to_user"]
unsafe fn switch_to_user(mut stack_ptr: *mut u32, process_regs: *mut __callee_context) -> *mut u32 {
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
        // after a SVC called by the user process
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
        out("r2") _, out("r3") _, out("r4") _, out("r5") _, out("r8") _, out("r10") _,
        out("r11") _, out("r12") _,
    );

    stack_ptr
}

// N: Maximum number of threads supported
// F: systick frequency (Hz)
#[repr(C)]
pub struct Kernel<'a, const F: u32 = 1> {
    tasks: list::List<'a, Thread<'a>>,
    count: usize,
    current: usize,

    // Ticks counter: period: P (ms)
    ticks: u64,
}

impl<'a, const F: u32> Kernel<'a, F> {
    pub const fn init() -> Kernel<'a, F> {
        Kernel {
            tasks: list::List::empty(),
            count: 0, // main thread
            current: 0,
            ticks: 0,
        }
    }

    pub fn register_thread(&mut self, thread: &'a Thread<'a>) {
        self.tasks.push_front(&thread);
        self.count += 1;
    }

    pub fn print_tasks(&self) {
        println!("print_tasks (cur: {} count: {})", self.current, self.count);
        for task in self.tasks.iter() {
            println!("{}", task);
        }
    }

    pub fn current(&self) -> &'a Thread<'a> {
        for (index, task) in self.tasks.iter().enumerate() {
            if self.current == index {
                return task;
            }
        }
        panic!("Invalid current index");
    }

    pub fn current_ptr(&self) -> *mut Thread {
        let current = self.current();
        current as *const Thread as *mut Thread
    }

    // TODO: Remove the Cs parameter, access to Kernel is already atomic
    pub fn increment_ticks(&mut self, _cs: &Cs<GlobalIrq>) {
        self.ticks += 1;
    }

    // TODO: Remove the Cs parameter, access to Kernel is already atomic
    pub fn get_ticks(&self, _cs: &Cs<GlobalIrq>) -> u64 {
        self.ticks
    }

    // TODO: Remove the cs, access to Kernel is already atomic
    pub fn busy_wait(&self, ms: u32) {
        let end = atomic_restore(|cs| self.get_ticks(cs)).saturating_add(((ms * F) / 1000) as u64);
        while atomic_restore(|cs| self.get_ticks(cs)) < end {}
    }

    pub fn sched_next_thread(&mut self) {
        self.current = (self.current + 1) % self.count;
    }

    pub fn kernel_loop(&mut self) {
        let task = self.current();

        let process_sp = task.stack_ptr.get();
        println!("PSP: 0x{}", Hex::U32(process_sp as u32));

        let process_context = task.context.as_ptr();

        let process_sp = unsafe { switch_to_user(process_sp, process_context) };

        stdio::write_bytes(&[b'!']);

        println!("PSP: 0x{}", Hex::U32(process_sp as u32));

        task.stack_ptr.set(process_sp);

        println!("Returned from switch_to_user");
    }
}
