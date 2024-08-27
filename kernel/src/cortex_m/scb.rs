/*
 * Table B3-8 NVIC register summary from ARM v7-M Architecture Reference Manual
 *
 * +----------------------+------------------------+------+----------------+---------------------------------------------------------+
 * | Address              | Name                   | Type | Reset          | Description                                             |
 * +======================+========================+======+================+=========================================================+
 * | 0xE000E100-          | NVIC_ISER0-            | RW   | 0x00000000     | Interrupt Set-Enable Registers, NVIC_ISER0-NVIC_ISER15  |
 * | 0xE000E13C           | NVIC_ISER15            |      |                | on page B3-684                                          |
 * +----------------------+------------------------+------+----------------+---------------------------------------------------------+
 * | 0xE000E180-          | NVIC_ICER0-            | RW   | 0x00000000     | Interrupt Clear-Enable Registers, NVIC_ICER0-NVIC_ICER15|
 * | 0xE000E1BC           | NVIC_ICER15            |      |                | on page B3-684                                          |
 * +----------------------+------------------------+------+----------------+---------------------------------------------------------+
 * | 0xE000E200-          | NVIC_ISPR0-            | RW   | 0x00000000     | Interrupt Set-Pending Registers, NVIC_ISPR0-NVIC_ISPR15 |
 * | 0xE000E23C           | NVIC_ISPR15            |      |                | on page B3-685                                          |
 * +----------------------+------------------------+------+----------------+---------------------------------------------------------+
 * | 0xE000E280-          | NVIC_ICPR0-            | RW   | 0x00000000     | Interrupt Clear-Pending Registers, NVIC_ICPR0-NVIC_ICPR15 |
 * | 0xE000E2BC           | NVIC_ICPR15            |      |                | on page B3-685                                          |
 * +----------------------+------------------------+------+----------------+---------------------------------------------------------+
 * | 0xE000E300-          | NVIC_IABR0-            | RO   | 0x00000000     | Interrupt Active Bit Registers, NVIC_IABR0-NVIC_IABR15  |
 * | 0xE000E33C           | NVIC_IABR15            |      |                | on page B3-686                                          |
 * +----------------------+------------------------+------+----------------+---------------------------------------------------------+
 * | 0xE000E340-          | -                      | -    | -              | Reserved                                                |
 * | 0xE000E3FC           |                        |      |                |                                                         |
 * +----------------------+------------------------+------+----------------+---------------------------------------------------------+
 * | 0xE000E400-          | NVIC_IPR0-             | RW   | 0x00000000     | Interrupt Priority Registers, NVIC_IPR0-NVIC_IPR123     |
 * | 0xE000E52C           | NVIC_IPR123            |      |                | on page B3-686                                          |
 * +----------------------+------------------------+------+----------------+---------------------------------------------------------+
 * | 0xE000E5F0-          | -                      | -    | -              | Reserved                                                |
 * | 0xE000E6FC           |                        |      |                |                                                         |
 * +----------------------+------------------------+------+----------------+---------------------------------------------------------+
*/

use core::ops::Deref;

use volatile_register::{RO, RW, WO};

use super::{irqn::SysIrqn, nvic::NVIC};

#[repr(C)]
pub struct SCBRegs {
    /// Offset: 0x000 (R/ )  CPUID Base Register
    cpuid: RO<u32>,
    /// Offset: 0x004 (R/W)  Interrupt Control and State Register
    icsr: RW<u32>,
    /// Offset: 0x008 (R/W)  Vector Table Offset Register
    vtor: RW<u32>,
    /// Offset: 0x00C (R/W)  Application Interrupt and Reset Control Register
    aircr: RW<u32>,
    /// Offset: 0x010 (R/W)  System Control Register
    scr: RW<u32>,
    /// Offset: 0x014 (R/W)  Configuration Control Register
    ccr: RW<u32>,
    /// Offset: 0x018 (R/W)  System Handlers Priority Registers (4-7, 8-11, 12-15)
    shp: [RW<u8>; 12],
    /// Offset: 0x024 (R/W)  System Handler Control and State Register
    shcsr: RW<u32>,
    /// Offset: 0x028 (R/W)  Configurable Fault Status Register
    cfsr: RW<u32>,
    /// Offset: 0x02C (R/W)  HardFault Status Register
    hfsr: RW<u32>,
    /// Offset: 0x030 (R/W)  Debug Fault Status Register
    dfsr: RW<u32>,
    /// Offset: 0x034 (R/W)  MemManage Fault Address Register
    mmfar: RW<u32>,
    /// Offset: 0x038 (R/W)  BusFault Address Register
    bfar: RW<u32>,
    /// Offset: 0x03C (R/W)  Auxiliary Fault Status Register
    afsr: RW<u32>,
    /// Offset: 0x040 (R/ )  Processor Feature Register
    pfr: [RO<u32>; 2],
    /// Offset: 0x048 (R/ )  Debug Feature Register
    dfr: RO<u32>,
    /// Offset: 0x04C (R/ )  Auxiliary Feature Register
    adr: RO<u32>,
    /// Offset: 0x050 (R/ )  Memory Model Feature Register
    mmfr: [RO<u32>; 4],
    /// Offset: 0x060 (R/ )  Instruction Set Attributes Register
    isar: [RO<u32>; 5],
    _reserved0: RW<u32>,
    /// Offset: 0x088 (R/W)  Coprocessor Access Control Register
    cpacr: RW<u32>,
}

pub struct SCB {}

impl SCB {
    pub const PTR: *const SCBRegs = 0xE000_ED00 as *const SCBRegs;

    pub fn new() -> Self {
        SCB {}
    }

    pub fn get_cpuid(&self) -> u32 {
        (*self).cpuid.read()
    }

    // Valid from SYSTICK(-1) to MEMORYMANAGEMENT(-12)
    pub fn get_priority(&self, irqn: SysIrqn) -> u8 {
        let index = ((irqn as u32) & 0xf) - 4;
        (*self).shp[index as usize].read() >> (8 - NVIC::PRIO_BITS)
    }

    // Valid from SYSTICK(-1) to MEMORYMANAGEMENT(-12)
    pub fn set_priority(&mut self, irqn: SysIrqn, prio: u8) {
        let index = ((irqn as u32) & 0xf) - 4;
        unsafe { (*self).shp[index as usize].write(prio << (8 - NVIC::PRIO_BITS)) }
    }
}

impl Deref for SCB {
    type Target = SCBRegs;

    #[inline(always)]
    fn deref(&self) -> &SCBRegs {
        unsafe { &*Self::PTR }
    }
}
