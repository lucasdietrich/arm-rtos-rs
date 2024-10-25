use crate::kernel::{thread::Thread, CpuVariant};

use super::{SwapData, SyncPrimitive};

impl Into<SwapData> for u32 {
    fn into(self) -> SwapData {
        SwapData::Signal(self)
    }
}

impl TryFrom<SwapData> for u32 {
    type Error = SwapData;

    fn try_from(swap: SwapData) -> Result<u32, SwapData> {
        match swap {
            SwapData::Signal(value) => Ok(value),
            _ => Err(swap),
        }
    }
}

pub struct Signal {
    value: Option<u32>,
}

impl Signal {
    pub const fn new() -> Self {
        Signal { value: None }
    }
}

impl<'a, CPU: CpuVariant> SyncPrimitive<'a, CPU> for Signal {
    type Swap = u32;

    fn release(&mut self, notify_value: u32) -> Result<(), u32> {
        self.value = Some(notify_value);

        Ok(())
    }

    fn acquire(&mut self, _thread: &'a Thread<'a, CPU>) -> Option<u32> {
        self.value
    }
}

impl Signal {
    pub fn reset(&mut self) {
        self.value = None;
    }
}
