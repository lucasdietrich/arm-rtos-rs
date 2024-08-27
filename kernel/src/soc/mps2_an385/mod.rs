pub mod uart;

pub use uart::{Uart, UartDevice};

use crate::cortex_m::systick::SysTick;

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
