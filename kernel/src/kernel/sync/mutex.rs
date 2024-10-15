use crate::kernel::{thread::Thread, CpuVariant};

use super::{SwapData, SyncPrimitiveTrait};

pub struct Ownership;

impl Into<SwapData> for Ownership {
    fn into(self) -> SwapData {
        SwapData::Ownership
    }
}

pub struct Mutex<'a, CPU: CpuVariant> {
    owner: Option<&'a Thread<'a, CPU>>,
}

impl<'a, CPU: CpuVariant> SyncPrimitiveTrait<'a, CPU> for Mutex<'a, CPU> {
    type Swap = Ownership;

    fn sync(&mut self, _notify_value: Ownership) {
        self.owner = None
    }

    fn pend(&mut self, thread: &'a Thread<'a, CPU>) -> Option<Ownership> {
        if self.owner.is_none() {
            self.owner = Some(thread);
            Some(Ownership)
        } else {
            None
        }
    }
}
