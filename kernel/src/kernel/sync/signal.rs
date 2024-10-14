use crate::kernel::{thread::Thread, CpuVariant};

use super::SyncPrimitiveTrait;

pub struct Signal {
    value: Option<u32>,
}

impl<'a, CPU: CpuVariant> SyncPrimitiveTrait<'a, CPU> for Signal {
    type Notify = u32;

    fn sync(&mut self, _thread: Option<&'a Thread<'a, CPU>>, notify_value: u32) {
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
