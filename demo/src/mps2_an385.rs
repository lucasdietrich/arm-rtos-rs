use volatile_register::RW;

use crate::device::{SerialConfig, SerialTrait};

/// Universal Asynchronous Receiver Transmitter (UART)
#[repr(C)]
pub struct Uart {
    /// Offset: 0x000 (R/W) Data Register
    pub data: RW<u32>,
    // Offset: 0x004 (R/W) Status Register
    pub state: RW<u32>,
    // Offset: 0x008 (R/W) Control Register
    pub ctrl: RW<u32>,
    // Offset: 0x00C (R/ ) Interrupt Status Register
    // Offset: 0x00C ( /W) Interrupt Clear Register
    pub int: RW<u32>,
    // Offset: 0x010 (R/W) Baudrate Divider Register
    pub bauddiv: RW<u32>,
}

pub const APB_BASE: usize = 0x4000_0000;

pub const UART0_BASE: usize = APB_BASE + 0x4000;
pub const UART1_BASE: usize = APB_BASE + 0x5000;
pub const UART2_BASE: usize = APB_BASE + 0x6000;
pub const UART3_BASE: usize = APB_BASE + 0x7000;
pub const UART4_BASE: usize = APB_BASE + 0x9000;

pub const UART0: *mut Uart = UART0_BASE as *mut Uart;
pub const UART1: *mut Uart = UART1_BASE as *mut Uart;
pub const UART2: *mut Uart = UART2_BASE as *mut Uart;
pub const UART3: *mut Uart = UART3_BASE as *mut Uart;
pub const UART4: *mut Uart = UART4_BASE as *mut Uart;

pub struct UartDevice<const FCPU: u32> {
    uart: *mut Uart,
}

impl<const FCPU: u32> UartDevice<FCPU> {
    pub fn new(uart: *mut Uart) -> Self {
        UartDevice { uart }
    }
}

impl<const FCPU: u32> SerialTrait for UartDevice<FCPU> {
    fn init(&self, config: SerialConfig) {
        unsafe { (*self.uart).bauddiv.write(FCPU / config.baudrate) } // Set baudrate
        unsafe { (*self.uart).ctrl.write(0x03) } // Enable RX and TX
    }

    fn write(&self, c: u8) {
        while unsafe { (*self.uart).state.read() & 0x1 } != 0 {}
        unsafe { (*self.uart).data.write(c as u32) }
    }

    fn read(&self) -> Option<u8> {
        let state = unsafe { (*self.uart).state.read() };

        if state & 0x2 != 0 {
            let data = unsafe { (*self.uart).data.read() };
            Some(data as u8)
        } else {
            None
        }
    }
}
