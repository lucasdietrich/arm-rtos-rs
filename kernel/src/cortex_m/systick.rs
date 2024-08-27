use core::ops::Deref;

use volatile_register::RW;

use super::SCS_BASE;

pub const SYSTICK_BASE: usize = SCS_BASE + 0x0010;
pub const SYSTICK: *mut SysTick = SYSTICK_BASE as *mut SysTick;

/// Universal Asynchronous Receiver Transmitter (UART)
#[repr(C)]
pub struct SysTick {
    /// Offset: 0x000 (R/W)  SysTick Control and Status Register
    pub ctrl: RW<u32>,
    /// Offset: 0x004 (R/W)  SysTick Reload Value Register
    pub load: RW<u32>,
    /// Offset: 0x008 (R/W)  SysTick Current Value Register
    pub val: RW<u32>,
    /// Offset: 0x00C (R/ )  SysTick Calibration Register
    pub calib: RW<u32>,
}

pub struct SysTickDevice<const FCPU: u32>;

const ENABLE_POS: u32 = 0;
const TICKINT_POS: u32 = 1;
const CLKSOURCE_POS: u32 = 2;

impl<const FCPU: u32> SysTickDevice<FCPU> {
    pub const PTR: *const SysTick = SYSTICK as *const _;

    #[inline(always)]
    pub fn instance() -> Self {
        SysTickDevice {}
    }

    pub fn configure<const FSYSCLOCK: u32>(&mut self, interrupt: bool) {
        const SOURCE: u32 = 1 << CLKSOURCE_POS;
        const ENABLE: u32 = 1 << ENABLE_POS;

        let tickint: u32 = if interrupt { 1 << TICKINT_POS } else { 0 };

        unsafe {
            (*self).load.write(FCPU / FSYSCLOCK);
            (*self).val.write(0);
            (*self).ctrl.write(SOURCE | ENABLE | tickint);
        }
    }
}

impl<const FCPU: u32> Deref for SysTickDevice<FCPU> {
    type Target = SysTick;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}
