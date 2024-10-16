use crate::{
    kernel::{
        thread::{Thread, Waitqueue},
        CpuVariant,
    },
    list,
};

use super::SyncPrimitive;

pub enum SwapData {
    Empty,
    Signal(u32),
    Ownership,
}

pub struct KernelObject<'a, S: SyncPrimitive<'a, CPU>, CPU: CpuVariant> {
    identifier: u32,
    waitqueue: list::List<'a, Thread<'a, CPU>, Waitqueue>,
    primitive: S,
}

impl<'a, S: SyncPrimitive<'a, CPU>, CPU: CpuVariant> KernelObject<'a, S, CPU> {
    pub fn new(identifier: u32, primitive: S) -> Self {
        KernelObject {
            identifier: identifier,
            waitqueue: list::List::empty(),
            primitive,
        }
    }

    pub fn release(&mut self, swap: S::Swap) -> Option<&'a Thread<'a, CPU>> {
        let unpended_thread = self.waitqueue.pop_head();
        if let Some(thread) = unpended_thread {
            thread.unpend(swap.into())
        } else {
            self.primitive.release(swap);
        }
        unpended_thread
    }

    pub fn acquire(
        &mut self,
        thread: &'a Thread<'a, CPU>,
        timeout_instant: Option<u64>,
    ) -> Option<S::Swap> {
        let obtained = self.primitive.acquire(thread);
        if obtained.is_none() {
            // If the object is not available, make the thread pending on it by
            // appending it at the end of the waitqueue.
            self.waitqueue.push_back(thread);

            // Mark the thread pending until the given instant
            thread.set_pending(self.identifier, timeout_instant);
        }
        obtained
    }
}
