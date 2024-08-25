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

pub struct SysTickDevice<const FCPU: u32> {
    device: *mut SysTick,
}

impl<const FCPU: u32> SysTickDevice<FCPU> {
    pub fn new(device: *mut SysTick) -> Self {
        SysTickDevice { device }
    }

    pub fn configure(&mut self, period: u32, interrupt: bool) {
        const SOURCE: u32 = 1 << 2;
        const ENABLE: u32 = 1 << 0;

        let tickint: u32 = if interrupt { 1 << 1 } else { 0 };

        unsafe {
            (*self.device).load.write(FCPU / period);
            (*self.device).val.write(0);
            (*self.device).ctrl.write(SOURCE | ENABLE | tickint);
        }
    }
}
