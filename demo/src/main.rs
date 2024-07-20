#![no_std]
#![no_main]
#![feature(stdarch_arm_hints)]
#![feature(stdarch_arm_neon_intrinsics)]
#![feature(core_intrinsics)]

use core::intrinsics;

use device::{SerialConfig, SerialTrait};
use mps2_an385::{UartDevice, UART0};
mod cortex_m;
mod device;

#[cfg(feature = "mps2-an385")]
mod mps2_an385;

static RO_MYVAR: u32 = 10;
static mut DATA_MYVAR: u32 = 10;

/* Explicitly place the var in the .bss section
 * but this is not necessary, as all 0-initialized
 * vars are placed in the .bss section by default */
#[link_section = ".bss"]
static mut BSS_MYVAR: u32 = 0;

const FCPU: u32 = 25_000_000;

pub fn _start() {
    let uart = UartDevice::<FCPU>::new(UART0);
    uart.init(SerialConfig::default());
    uart.write_str("Hello, world!\n");

    loop {
        if let Some(byte) = uart.read() {
            uart.write_str("data: ");
            uart.write(byte);
            uart.write(b'\n');
        }
    }

    unsafe {
        BSS_MYVAR = 1;
    }

    loop {
        unsafe {
            DATA_MYVAR = DATA_MYVAR.wrapping_sub(BSS_MYVAR);
        }

        if unsafe { DATA_MYVAR } == 0 {
            break;
        }
    }

    intrinsics::abort();
}
