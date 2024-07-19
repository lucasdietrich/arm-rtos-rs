#![no_std]
#![no_main]
#![feature(stdarch_arm_hints)]
#![feature(stdarch_arm_neon_intrinsics)]

mod cortex_m;
mod mps2_an385;

#[no_mangle]
pub extern "C" fn _start() {
    let _x = 0xAAAAAAAA_u32;
}
