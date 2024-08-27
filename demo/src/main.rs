#![no_std]
#![no_main]
#![feature(stdarch_arm_hints)]
#![feature(stdarch_arm_neon_intrinsics)]
#![feature(core_intrinsics)]
#![feature(maybe_uninit_uninit_array)]

mod cortex_m_rt;
mod critical_section;
mod entry;
mod errno;
mod io;
mod kernel;
mod serial;
mod serial_utils;
mod syscalls;
mod systick;
mod threading;
mod userspace;

#[cfg(feature = "mps2-an385")]
mod mps2_an385;
