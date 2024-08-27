#![no_std]
#![feature(stdarch_arm_hints)]
#![feature(stdarch_arm_neon_intrinsics)]
#![feature(core_intrinsics)]
#![feature(maybe_uninit_uninit_array)]

pub mod list;

pub use list::*;

pub fn test() {}
