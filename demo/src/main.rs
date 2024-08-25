#![no_std]
#![no_main]
#![feature(stdarch_arm_hints)]
#![feature(stdarch_arm_neon_intrinsics)]
#![feature(core_intrinsics)]
#![feature(maybe_uninit_uninit_array)]

mod cortex_m_rt;
mod entry;
mod io;
mod kernel;
mod serial;
mod serial_utils;
mod systick;
mod task;
mod threading;
mod userspace;

#[cfg(feature = "mps2-an385")]
mod mps2_an385;
