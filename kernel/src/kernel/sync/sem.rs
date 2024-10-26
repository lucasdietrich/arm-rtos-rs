use crate::kernel::{sync::SyncPrimitive, thread::Thread, CpuVariant};

use super::traits::ReleaseOutcome;

pub struct Semaphore {
    cur: u32,
    max: u32,
}

impl Semaphore {
    pub const fn new(init: u32, max: u32) -> Self {
        Semaphore { cur: init, max }
    }
}

impl<'a, CPU: CpuVariant> SyncPrimitive<'a, CPU> for Semaphore {
    type Swap = ();

    fn release(&mut self, _released: ()) -> Result<ReleaseOutcome<()>, ()> {
        self.cur = (self.cur + 1).max(self.max);

        Ok(ReleaseOutcome::Released)
    }

    fn acquire(&mut self, _thread: &'a Thread<'a, CPU>) -> Option<()> {
        if self.cur == 0 {
            None
        } else {
            self.cur -= 1;
            Some(())
        }
    }
}
