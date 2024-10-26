use core::ops::Deref;

use volatile_register::RW;

use super::SCS_BASE;

pub const SYSTICK_BASE: usize = SCS_BASE + 0x0010;
pub const SYSTICK: *mut SysTickRegs = SYSTICK_BASE as *mut SysTickRegs;

/// Universal Asynchronous Receiver Transmitter (UART)
#[repr(C)]
pub struct SysTickRegs {
    /// Offset: 0x000 (R/W)  SysTick Control and Status Register
    pub ctrl: RW<u32>,
    /// Offset: 0x004 (R/W)  SysTick Reload Value Register
    pub load: RW<u32>,
    /// Offset: 0x008 (R/W)  SysTick Current Value Register
    pub val: RW<u32>,
    /// Offset: 0x00C (R/ )  SysTick Calibration Register
    pub calib: RW<u32>,
}

pub struct SysTick<const FREQ_SYS_TICK: u32>;

const ENABLE_POS: u32 = 0;
const TICKINT_POS: u32 = 1;
const CLKSOURCE_POS: u32 = 2;
const COUNTFLAG_POS: u32 = 16;

impl<const FREQ_SYS_TICK: u32> SysTick<FREQ_SYS_TICK> {
    pub const PTR: *const SysTickRegs = SYSTICK as *const _;

    pub fn configure_period<const FCPU: u32>(interrupt: bool) -> SysTick<FREQ_SYS_TICK> {
        const SOURCE: u32 = 1 << CLKSOURCE_POS;
        const ENABLE: u32 = 1 << ENABLE_POS;

        let tickint: u32 = if interrupt { 1 << TICKINT_POS } else { 0 };

        unsafe {
            Self.load.write(FCPU / FREQ_SYS_TICK);
            Self.val.write(0);
            Self.ctrl.write(SOURCE | ENABLE | tickint);
        }

        SysTick
    }

    pub fn get_reload_value(&self) -> u32 {
        self.load.read()
    }

    pub fn get_current_value(&self) -> u32 {
        self.val.read()
    }

    pub fn get_countflag(&self) -> bool {
        self.ctrl.read() & (1 << COUNTFLAG_POS) != 0
    }
}

impl<const FREQ_SYS_TICK: u32> Deref for SysTick<FREQ_SYS_TICK> {
    type Target = SysTickRegs;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}
