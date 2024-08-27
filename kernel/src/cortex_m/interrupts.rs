use core::{
    arch::asm,
    sync::atomic::{compiler_fence, Ordering},
};

use super::critical_section::{self, Cs, GlobalIrq};

pub fn primask() -> bool {
    let primask: u32;

    // Read section B5.2.3 of ARM v7-M Architecture Reference Manual
    unsafe { asm!("mrs {}, PRIMASK", out(reg) primask, options(nostack, nomem, preserves_flags)) }

    primask & (1 << 0) == 0
}

pub fn disable() -> Cs<GlobalIrq> {
    unsafe {
        asm!("cpsid i");
    }

    /* TODO reason why this is before for disable() and after for enable() */
    /* Force memory operations termination and enforce instructions ordering,
     * ensure memory operations completed once interrupts are disabled */
    compiler_fence(Ordering::SeqCst);

    /* TODO: review design
     * Make enable and disable very coupled, i.e:
     * impossible to enable interrupts if not leaving a critical section,
     * initiated by disable()
     */
    unsafe { Cs::<GlobalIrq>::new() }
}

pub fn enable(_cs: Cs<GlobalIrq>) {
    /* Force memory operations termination and enforce instructions ordering,
     * memory operations must complete before enabling interrupts */
    compiler_fence(Ordering::SeqCst);

    unsafe {
        asm!("cpsie i");
    }
}

// Reference implementation: https://github.com/rust-embedded/cortex-m/blob/master/cortex-m/src/interrupt.rs
pub fn atomic_section<const FORCEON: bool, F, R>(f: F) -> R
where
    F: FnOnce(&Cs<critical_section::GlobalIrq>) -> R,
{
    /* Get if interrupts are enabled */
    let primask = FORCEON || primask();

    /* Disable interrupts and execute the close */
    let cs = disable();

    let ret = f(&cs);

    /* Reenable interrupts if they were enable,
     * otherwise keep them disable */
    if primask {
        enable(cs);
    }

    ret
}
