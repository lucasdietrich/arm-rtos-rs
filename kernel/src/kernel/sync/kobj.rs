use crate::{
    kernel::{thread::Thread, timeout::Timeout, CpuVariant},
    list,
};

use super::SyncPrimitiveTrait;

pub enum SwapData {
    Empty,
    Signal(u32),
    Ownership,
}

pub struct KernelObject<'a, S: SyncPrimitiveTrait<'a, CPU>, CPU: CpuVariant> {
    identifier: u32,
    waitqueue: list::List<'a, Thread<'a, CPU>>,
    primitive: S,
}

impl<'a, S: SyncPrimitiveTrait<'a, CPU>, CPU: CpuVariant> KernelObject<'a, S, CPU> {
    pub fn new(identifier: u32, primitive: S) -> Self {
        KernelObject {
            identifier: identifier,
            waitqueue: list::List::empty(),
            primitive,
        }
    }

    pub fn sync(&mut self, swap: S::Swap) -> Option<&'a Thread<'a, CPU>> {
        let unpended_thread = self.waitqueue.pop_head();
        if let Some(thread) = unpended_thread {
            thread.unpend(swap.into())
        } else {
            self.primitive.sync(swap);
        }
        unpended_thread
    }

    pub fn pend(
        &mut self,
        thread: &'a Thread<'a, CPU>,
        timeout_instant: Option<u64>,
    ) -> Option<S::Swap> {
        let result = self.primitive.pend(thread);
        if result.is_none() {
            self.waitqueue.push_back(thread);
            thread.set_pending(self.identifier, timeout_instant);
        }
        result
    }
}
