use crate::kernel::{thread::Thread, CpuVariant};

use super::{SwapData, SyncPrimitive};

impl Into<SwapData> for u32 {
    fn into(self) -> SwapData {
        SwapData::Signal(self)
    }
}

impl TryFrom<SwapData> for u32 {
    type Error = ();

    fn try_from(swap: SwapData) -> Result<u32, ()> {
        match swap {
            SwapData::Signal(value) => Ok(value),
            _ => Err(()),
        }
    }
}

pub struct Signal {
    value: Option<u32>,
}

impl<'a, CPU: CpuVariant> SyncPrimitive<'a, CPU> for Signal {
    type Swap = u32;

    fn release(&mut self, notify_value: u32) {
        self.value = Some(notify_value);
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
