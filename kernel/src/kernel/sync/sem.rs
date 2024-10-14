use crate::kernel::{sync::SyncPrimitiveTrait, thread::Thread, CpuVariant};

pub struct Semaphore {
    cur: u32,
    max: u32,
}

impl<'a, CPU: CpuVariant> SyncPrimitiveTrait<'a, CPU> for Semaphore {
    type Notify = ();

    fn sync(&mut self, thread: Option<&'a Thread<'a, CPU>>, _notify_value: ()) {
        if thread.is_none() {
            self.cur = (self.cur + 1).max(self.max)
        }
    }

    fn pend(&mut self, _thread: &'a Thread<'a, CPU>) -> Option<()> {
        if self.cur == 0 {
            None
        } else {
            self.cur = self.cur - 1;
            Some(())
        }
    }
}
