#![no_std]
#![no_main]
#![feature(stdarch_arm_hints)]
#![feature(stdarch_arm_neon_intrinsics)]
#![feature(core_intrinsics)]
#![feature(maybe_uninit_uninit_array)]

mod entry;

pub unsafe fn ref_cast_lifetime<'a, T: 'a>(val: &'a T) -> &'static T {
    unsafe { &*(val as *const T) }
}
