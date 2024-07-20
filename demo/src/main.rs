#![no_std]
#![no_main]
#![feature(stdarch_arm_hints)]
#![feature(stdarch_arm_neon_intrinsics)]

mod cortex_m;
mod mps2_an385;

static RO_MYVAR: u32 = 10;
static mut DATA_MYVAR: u32 = 10;
static mut BSS_MYVAR: u32 = 0;

#[no_mangle]
pub extern "C" fn _start() {
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
}
