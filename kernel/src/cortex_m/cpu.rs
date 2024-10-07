use core::{arch::asm, mem::MaybeUninit};

use crate::println;

pub struct Cpu;

impl Cpu {
    #[inline(never)]
    unsafe fn registers_fill(regs: *mut [u32; 16]) {
        unsafe {
            let ptr = regs as *mut u32;

            asm!(
                "stmia r0, {{r0 - r12}}",

                // Store sp (r13)
                "str sp, [r0, #0x34]",
                // Store lr (r14)
                "str lr, [r0, #0x38]",
                // Store pc (r15)
                "adr r1, .", // Get current address into r1
                "str r1, [r0, #0x3c]",
                inout("r0") ptr => _,
                options(nostack, preserves_flags),
            );
        }
    }

    pub fn registers() -> [u32; 16] {
        let mut regs = MaybeUninit::<[u32; 16]>::uninit();

        let regs_ptr = regs.as_mut_ptr();

        unsafe {
            Self::registers_fill(regs_ptr);

            regs.assume_init()
        }
    }

    // WARNING: Very stack consuming without optimizations
    pub fn print_registers(regs: &[u32; 16]) {
        println!("Registers:");
        println!(
            "r0: 0x{:08x} r1: 0x{:08x} r2: 0x{:08x} r3: 0x{:08x}",
            regs[0], regs[1], regs[2], regs[3]
        );
        println!(
            "r4: 0x{:08x} r5: 0x{:08x} r6: 0x{:08x} r7: 0x{:08x}",
            regs[4], regs[5], regs[6], regs[7]
        );
        println!(
            "r8: 0x{:08x} r9: 0x{:08x} r10: 0x{:08x} r11: 0x{:08x}",
            regs[8], regs[9], regs[10], regs[11]
        );
        println!(
            "r12: 0x{:08x} sp: 0x{:08x} lr: 0x{:08x} pc: 0x{:08x}",
            regs[12], regs[13], regs[14], regs[15]
        );
    }
}
