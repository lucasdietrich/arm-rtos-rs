use crate::cortex_m_rt::FCPU;
use crate::{mps2_an385::UartDevice, serial::SerialTrait};
use core::fmt::{Arguments, Write};

static mut STDIO_UART: Option<UartDevice<FCPU>> = None;

pub fn set_uart(uart: UartDevice<FCPU>) {
    unsafe { STDIO_UART = Some(uart) };
}

pub fn read() -> Option<u8> {
    if let Some(uart) = unsafe { STDIO_UART.as_ref() } {
        uart.read()
    } else {
        None
    }
}

pub fn write_bytes(bytes: &[u8]) {
    if let Some(uart) = unsafe { STDIO_UART.as_mut() } {
        for byte in bytes {
            uart.write_byte(*byte)
        }
    }
}

pub fn write_args(args: Arguments<'_>, nl: bool) {
    if let Some(uart) = unsafe { STDIO_UART.as_mut() } {
        let _ = uart.write_fmt(args);

        if nl {
            uart.write_byte(b'\n');
        }
    }
}

#[macro_export]
macro_rules! print {
    () => {};
    ($($arg:tt)*) => {{
        $crate::io::write_args(format_args!($($arg)*), false)
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::io::write_args(format_args!("\n"), false)
    };
    ($($arg:tt)*) => {{
        $crate::io::write_args(format_args!($($arg)*), true)
    }}
}
