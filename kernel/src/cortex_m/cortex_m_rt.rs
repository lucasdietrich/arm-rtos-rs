use core::{
    intrinsics::{volatile_load, volatile_store},
    ptr::{self, addr_of, addr_of_mut},
};

use crate::kernel::{kernel::z_pendsv, syscalls::z_svc};

// TODO move to mps2_an385
pub const FCPU: u32 = 25_000_000;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn _default_handler() {
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn _unimplemented() {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "C" fn _fault_handler() {
    loop {}
}

/* External symbols defined in the application */
extern "C" {
    // Not a real function, but a symbol in the linker script
    // which points to the top of the stack
    fn _stack_top();

    /* TODO move to the current crate */
    fn z_systick();

    /* main of the application */
    fn _start();
}

#[no_mangle]
#[used]
#[link_section = ".vector_table"]
static VECTOR_TABLE: [unsafe extern "C" fn(); 16] = [
    // Initial Stack Pointer
    _stack_top,
    // Reset Handler
    _reset_handler,
    // NMI Handler
    _default_handler,
    // Hard Fault Handler
    _fault_handler,
    // MPU Fault Handler
    _fault_handler,
    // Bus Fault Handler
    _fault_handler,
    // Usage Fault Handler
    _fault_handler,
    // Reserved
    _unimplemented,
    // Reserved
    _unimplemented,
    // Reserved
    _unimplemented,
    // Reserved
    _unimplemented,
    // SVCall Handler
    z_svc,
    // Debug Monitor Handler
    _default_handler,
    // Reserved
    _unimplemented,
    // PendSV Handler
    z_pendsv,
    // SysTick Handler
    z_systick,
    // DEVICES INTERRUPTS
];

// Retrieve symbols from the linker script
extern "C" {
    static mut _sbss: u8;
    static mut _ebss: u8;

    static mut _sdata: u8;
    static mut _edata: u8;
    static _sidata: u8;
}

// This trick prevent to have two instances of the same code
// from cortex-m crate
#[export_name = "cortex init duplicate found: __ONCE__"]
pub static __ONCE__: () = ();

// TODO also mark as "-> !"
#[no_mangle]
pub unsafe extern "C" fn _reset_handler() {
    // Clear the .bss section
    let bss_size = addr_of!(_ebss) as usize - addr_of!(_sbss) as usize;
    ptr::write_bytes(addr_of_mut!(_ebss), 0, bss_size);

    // Copy the .data section from FLASH to RAM
    let data_size = addr_of!(_sdata) as usize - addr_of!(_sbss) as usize;
    ptr::copy_nonoverlapping(addr_of!(_sidata), addr_of_mut!(_sdata), data_size);

    // Call the entry point of the program
    _start();

    // Loop forever if _start returns
    loop {}
}

pub fn reg_modify(reg: *mut u32, val: u32, mask: u32) {
    unsafe {
        let reg_val = volatile_load(reg);
        volatile_store(reg, (reg_val & !mask) | val);
    }
}

// Triggering PendSV exception causes the state context to be saved on the stack
//
// Stack frame: (from top to bottom)
//  return program status register (xPSR)
//  return address
//  LR
//  R12
//  R3
//  R2
//  R1
//  R0
#[no_mangle]
pub unsafe extern "C" fn k_call_pendsv() {
    // This code is equivalent to
    // unsafe { p.SCB.icsr.modify(|r| r | 1 << 28) }; // SCB->ICSR |= SCB_ICSR_PENDSVSET_Msk;

    const ICSR: *mut u32 = 0xE000_ED04 as *mut u32;
    const PENDSVSET_BIT: u32 = 1 << 28;
    reg_modify(ICSR, PENDSVSET_BIT, PENDSVSET_BIT);
}

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
