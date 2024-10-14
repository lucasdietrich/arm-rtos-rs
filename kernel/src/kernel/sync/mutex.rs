use crate::kernel::{thread::Thread, CpuVariant};

use super::SyncPrimitiveTrait;

pub struct Mutex<'a, CPU: CpuVariant> {
    owner: Option<&'a Thread<'a, CPU>>,
}

impl<'a, CPU: CpuVariant> SyncPrimitiveTrait<'a, CPU> for Mutex<'a, CPU> {
    type Notify = ();

    fn sync(&mut self, thread: Option<&'a Thread<'a, CPU>>, _notify_value: ()) {
        self.owner = thread
    }

    fn pend(&mut self, thread: &'a Thread<'a, CPU>) -> Option<()> {
        if self.owner.is_none() {
            self.owner = Some(thread);
            Some(())
        } else {
            None
        }
    }
}
