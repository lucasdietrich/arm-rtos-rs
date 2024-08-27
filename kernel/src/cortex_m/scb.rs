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

    #[inline(always)]
    pub fn instance() -> Self {
        SCB {}
    }

    pub fn get_cpuid(&self) -> u32 {
        (*self).cpuid.read()
    }

    // Valid from SYSTICK(-1) to MEMORYMANAGEMENT(-12)
    // prio can be between 0 and 7
    pub fn get_priority(&self, irqn: SysIrqn) -> u8 {
        let index = ((irqn as u32) & 0xf) - 4;
        (*self).shp[index as usize].read() >> (8 - NVIC::PRIO_BITS)
    }

    // Valid from SYSTICK(-1) to MEMORYMANAGEMENT(-12)
    // prio can be between 0 and 7
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
