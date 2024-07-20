use core::ptr::{self, addr_of, addr_of_mut};

use crate::_start;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
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

// Retrieve symbols from the linker script
extern "C" {
    static mut _sbss: u8;
    static mut _ebss: u8;

    static mut _sdata: u8;
    static mut _edata: u8;
    static _sidata: u8;
}

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
