use crate::kernel::{sync::SyncPrimitive, thread::Thread, CpuVariant};

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

    fn release(&mut self, _released: ()) {
        self.cur = (self.cur + 1).max(self.max)
    }

    fn acquire(&mut self, _thread: &'a Thread<'a, CPU>) -> Option<()> {
        if self.cur == 0 {
            None
        } else {
            self.cur = self.cur - 1;
            Some(())
        }
    }
}
