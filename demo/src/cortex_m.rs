use core::arch::{arm, asm};

use crate::_start;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// TODO also mark as "-> !"
#[no_mangle]
pub unsafe extern "C" fn _reset_handler() {
    _start();

    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn _to_implement() {
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
    _to_implement,
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
    _to_implement,
    // Debug Monitor Handler
    _to_implement,
    // Reserved
    _unimplemented,
    // PendSV Handler
    _to_implement,
    // SysTick Handler
    _to_implement,
    // DEVICES INTERRUPTS
];
