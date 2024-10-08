use core::{
    intrinsics::{volatile_load, volatile_store},
    ptr::{self, addr_of, addr_of_mut},
};

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

    /* main of the application */
    fn _start();

    /* Various interrupt handlers implementation:
     * Systick, SVC and PendSV */
    fn z_systick();
    fn z_svc();
    fn z_pendsv();
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

    // Call the entry point of the program (it is the main !)
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

    // TODO This can probably be simplified to a write only, and not modify,
    // as writing 0 to all bits in ICSR has no effect.
    // So following assembly instructions must be enough to trigger the pendsv:
    //
    // ldr r0, =0xE000ED04   ; Load ICSR address
    // ldr r1, =0x10000000   ; Load PENDSVSET bit value
    // str r1, [r0]          ; Trigger PendSV by writing to ICSR
    const ICSR: *mut u32 = 0xE000_ED04 as *mut u32;
    const PENDSVSET_BIT: u32 = 1 << 28;
    reg_modify(ICSR, PENDSVSET_BIT, PENDSVSET_BIT);
}
