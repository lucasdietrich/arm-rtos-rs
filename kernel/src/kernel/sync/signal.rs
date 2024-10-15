use crate::kernel::{thread::Thread, CpuVariant};

use super::{SwapData, SyncPrimitiveTrait};

impl Into<SwapData> for u32 {
    fn into(self) -> SwapData {
        SwapData::Signal(self)
    }
}

pub struct Signal {
    value: Option<u32>,
}

impl<'a, CPU: CpuVariant> SyncPrimitiveTrait<'a, CPU> for Signal {
    type Swap = u32;

    fn sync(&mut self, notify_value: u32) {
        self.value = Some(notify_value);
    }

    fn pend(&mut self, _thread: &'a Thread<'a, CPU>) -> Option<u32> {
        self.value
    }
}

impl Signal {
    pub fn reset(&mut self) {
        self.value = None;
    }
}
