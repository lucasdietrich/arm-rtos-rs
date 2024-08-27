use core::fmt;

use crate::println;

pub enum Hex {
    U8(u8),
    U16(u16),
    U32(u32),
}

impl fmt::Display for Hex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Hex::U8(byte) => write!(f, "0x{:02x}", byte),
            Hex::U16(half) => write!(f, "0x{:04x}", half),
            Hex::U32(word) => write!(f, "0x{:08x}", word),
        }
    }
}

#[no_mangle]
pub extern "C" fn z_debug(r0: u32) {
    println!("debug: {}", Hex::U32(r0))
}
