use crate::{mps2_an385::UartDevice, serial::SerialTrait};
use core::fmt::{Arguments, Write};

pub const FCPU: u32 = 25_000_000;

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

pub fn _print(args: Arguments<'_>, nl: bool) {
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
        $crate::io::_print(format_args!($($arg)*), false);
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::io::_print(format_args!("\n"), false);
    };
    ($($arg:tt)*) => {{
        $crate::io::_print(format_args!($($arg)*), true);
    }}
}
