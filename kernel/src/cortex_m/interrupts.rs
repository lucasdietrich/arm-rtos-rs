use core::{
    arch::asm,
    sync::atomic::{compiler_fence, Ordering},
};

pub fn primask() -> bool {
    let primask: u32;

    unsafe { asm!("mrs {}, PRIMASK", out(reg) primask, options(nostack, nomem, preserves_flags)) }

    primask & 1 != 0
}

pub fn disable() {
    unsafe {
        asm!("cpsid i");
    }

    /* TODO reason why this is before for disable() and after for enable() */
    /* Force memory operations termination and enforce instructions ordering,
     * ensure memory operations completed once interrupts are disabled */
    compiler_fence(Ordering::SeqCst);
}

pub fn enable() {
    /* Force memory operations termination and enforce instructions ordering,
     * memory operations must complete before enabling interrupts */
    compiler_fence(Ordering::SeqCst);

    unsafe {
        asm!("cpsie i");
    }
}

// Reference implementation: https://github.com/rust-embedded/cortex-m/blob/master/cortex-m/src/interrupt.rs
pub fn atomic_section<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    /* Get if interrupts are enabled */
    let primask = primask();

    /* TODO should produce as Cs here for f() ? */

    /* Disable interrupts and execute the close */
    disable();

    let ret = f();

    /* Reenable interrupts if they were enable,
     * otherwise keep them disable */
    if primask {
        enable();
    }

    ret
}
