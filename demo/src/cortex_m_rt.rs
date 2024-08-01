use core::{
    arch::{asm, global_asm},
    intrinsics::{volatile_load, volatile_store},
    ptr::{self, addr_of, addr_of_mut},
};

use crate::{_pendsv, _start, KERNEL};
use core::intrinsics;

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

#[no_mangle]
pub unsafe extern "C" fn _svc() {
    asm!("nop");
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
    _pendsv,
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
pub unsafe extern "C" fn pendsv_set() {
    // This code is equivalent to
    // unsafe { p.SCB.icsr.modify(|r| r | 1 << 28) }; // SCB->ICSR |= SCB_ICSR_PENDSVSET_Msk;

    let icsr = 0xE000_ED04 as *mut u32;
    let pendsvset_bit = 1 << 28;
    reg_modify(icsr, pendsvset_bit, pendsvset_bit);
}
