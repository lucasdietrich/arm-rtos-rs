use core::{
    arch::asm,
    ffi::c_void,
    intrinsics::{volatile_load, volatile_store},
    ptr::{self, addr_of, addr_of_mut},
};

// Deduplicate
pub const FCPU: u32 = 25_000_000;

use crate::{
    entry::{_start, _svc},
    kernel::z_pendsv,
};

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

extern "C" {
    // Not a real function, but a symbol in the linker script
    // which points to the top of the stack
    fn _stack_top();
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
    _svc,
    // Debug Monitor Handler
    _default_handler,
    // Reserved
    _unimplemented,
    // PendSV Handler
    z_pendsv,
    // SysTick Handler
    _default_handler,
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

#[no_mangle]
pub unsafe extern "C" fn enable_irq() {
    asm!("cpsid i");
}

#[no_mangle]
pub unsafe extern "C" fn disable_irq() {
    asm!("cpsie i");
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
pub unsafe extern "C" fn trig_pendsv() {
    // This code is equivalent to
    // unsafe { p.SCB.icsr.modify(|r| r | 1 << 28) }; // SCB->ICSR |= SCB_ICSR_PENDSVSET_Msk;

    const ICSR: *mut u32 = 0xE000_ED04 as *mut u32;
    const PENDSVSET_BIT: u32 = 1 << 28;
    reg_modify(ICSR, PENDSVSET_BIT, PENDSVSET_BIT);
}

#[no_mangle]
// Read A7.7.175 of DDI0403E_B_armv7m_arm.pdf
// TODO how to read back svc value 0xbb
// -> read pc-4
pub unsafe extern "C" fn trig_svc_alt(
    r0: *mut c_void,
    r1: *mut c_void,
    r2: *mut c_void,
    r3: *mut c_void,
) {
    asm!(
        "
        svc #0xbb
    ",
    in("r0") r0,
    in("r1") r1,
    in("r2") r2,
    in("r3") r3,

    // Indication:
    options(nostack, nomem),
    );
}

#[no_mangle]
pub unsafe extern "C" fn trig_svc_default() {
    trig_svc_alt(
        0xaaaaaaaa as *mut c_void,
        0xbbbbbbbb as *mut c_void,
        0xcccccccc as *mut c_void,
        0xdddddddd as *mut c_void,
    )
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
