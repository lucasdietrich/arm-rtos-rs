use volatile_register::RW;

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

pub struct SysTickDevice<const FCPU: u32, const PRIO: u32> {
    device: *mut SysTick,
}

const ENABLE_POS: u32 = 0;
const TICKINT_POS: u32 = 1;
const CLKSOURCE_POS: u32 = 2;

impl<const FCPU: u32, const PRIO: u32> SysTickDevice<FCPU, PRIO> {
    pub fn new(device: *mut SysTick) -> Self {
        SysTickDevice { device }
    }

    pub fn configure<const FSYSCLOCK: u32>(&mut self, interrupt: bool) {
        const SOURCE: u32 = 1 << CLKSOURCE_POS;
        const ENABLE: u32 = 1 << ENABLE_POS;

        let tickint: u32 = if interrupt { 1 << TICKINT_POS } else { 0 };

        unsafe {
            (*self.device).load.write(FCPU / FSYSCLOCK);
            (*self.device).val.write(0);
            (*self.device).ctrl.write(SOURCE | ENABLE | tickint);
        }
    }
}
