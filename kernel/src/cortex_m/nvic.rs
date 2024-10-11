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

use volatile_register::{RW, WO};

#[repr(C)]
pub struct NVICRegs {
    /// Interrupt Set Enable Register
    iser: [RW<u32>; 8],
    _reserved0: [u32; 24],
    /// Interrupt Clear Enable Register
    icer: [RW<u32>; 8],
    _reserved1: [u32; 24],
    /// Interrupt Set Pending Register
    ispr: [RW<u32>; 8],
    _reserved2: [u32; 24],
    /// Interrupt Clear Pending Register
    icpr: [RW<u32>; 8],
    _reserved3: [u32; 24],
    /// Interrupt Active bit Register
    iabr: [RW<u32>; 8],
    _reserved4: [u32; 56],
    /// Interrupt Priority Register
    ipr: [RW<u8>; 240],
    _reserved5: [u32; 644],
    /// Software Trigger Interrupt Register
    stir: WO<u32>,
}

pub struct NVIC;

impl NVIC {
    #[cfg(feature = "cm3")]
    pub const PRIO_BITS: u8 = 3;

    pub const PTR: *const NVICRegs = 0xE000_E100 as *const NVICRegs;

    pub fn instance() -> Self {
        NVIC {}
    }

    /* Reference implementation: __NVIC_GetPriority
     * https://github.com/ARM-software/CMSIS_5/blob/develop/CMSIS/Core/Include/core_cm3.h#L1694
     *
     * prio can be between 0 and 7
     */
    pub fn get_priority(&mut self, irqn: u16) -> u8 {
        let reg: &RW<u8> = &self.ipr[irqn as usize];
        reg.read() >> (8 - Self::PRIO_BITS)
    }

    /* Reference implementation: __NVIC_SetPriority
     * https://github.com/ARM-software/CMSIS_5/blob/develop/CMSIS/Core/Include/core_cm3.h#L1672
     *
     * prio can be between 0 and 7
     */
    pub fn set_priority(&mut self, irqn: u16, prio: u8) {
        let reg: &RW<u8> = &self.ipr[irqn as usize];
        unsafe { reg.write(prio << (8 - Self::PRIO_BITS)) }
    }
}

impl Deref for NVIC {
    type Target = NVICRegs;

    #[inline(always)]
    fn deref(&self) -> &NVICRegs {
        unsafe { &*Self::PTR }
    }
}
