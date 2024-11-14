use core::fmt::{Arguments, Write};

use crate::{cortex_m::cortex_m_rt::FCPU, serial::SerialTrait, soc::mps2_an38x::UartDevice};

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

// Write 16B per line
pub fn write_hex(val: &[u8]) {
    if let Some(uart) = unsafe { STDIO_UART.as_mut() } {
        for (i, byte) in val.iter().enumerate() {
            write!(uart, "{:02X} ", byte).unwrap();
            if i % 16 == 15 {
                uart.write_byte(b'\n');
            }
        }
        if val.len() % 16 != 0 {
            uart.write_byte(b'\n');
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
        $crate::stdio::write_args(format_args!($($arg)*), false)
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::stdio::write_args(format_args!("\n"), false)
    };
    ($($arg:tt)*) => {{
        $crate::stdio::write_args(format_args!($($arg)*), true)
    }}
}
