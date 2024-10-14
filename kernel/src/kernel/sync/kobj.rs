use core::default;

use crate::{
    kernel::{
        kernel::MS_PER_TICK,
        thread::{Thread, Waitqueue},
        timeout::Timeout,
        CpuVariant,
    },
    list::singly_linked as sl,
};

use super::{traits::Swappable, SyncPrimitive};

#[derive(Default, Debug)]
pub enum SwapData {
    #[default]
    Empty,
    Signal(u32),
    Ownership,
}

/// Implement a concrete synchronization primitive
pub struct KernelObject<'a, S: SyncPrimitive<'a, CPU>, CPU: CpuVariant> {
    /// Identifier of the kernel object
    identifier: u32,
    /// List of threads waiting on the kernel object
    waitqueue: sl::List<'a, Thread<'a, CPU>, Waitqueue>,
    /// The primitive implementation
    primitive: S,
}

impl<'a, S: SyncPrimitive<'a, CPU>, CPU: CpuVariant> KernelObject<'a, S, CPU> {
    /// Create a new kernel object
    pub fn new(identifier: u32, primitive: S) -> Self {
        KernelObject {
            identifier: identifier,
            waitqueue: sl::List::empty(),
            primitive,
        }
    }

    /// Notify the first thread in the waitqueue or release the primitive
    /// if no thread is waiting.
    pub fn notify_or_release(&mut self, swap: S::Swap) -> Option<&'a Thread<'a, CPU>> {
        let unpended_thread = self.waitqueue.pop_head();
        if let Some(thread) = unpended_thread {
            thread.unpend(swap.into())
        } else {
            self.primitive.release(swap);
        }
        unpended_thread
    }

    /// Make the thread try to acquire the primitive or make it pending
    /// for the given duration.
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

    pub fn remove_thread(&mut self, thread: &'a Thread<'a, CPU>) {
        self.waitqueue.remove(thread);
    }
}

pub trait KernelObjectTrait<'a, CPU: CpuVariant> {
    fn remove_thread(&mut self, thread: &'a Thread<'a, CPU>);
    fn acquire(
        &mut self,
        thread: &'a Thread<'a, CPU>,
        ticks: u64,
        timeout: Timeout,
    ) -> Option<SwapData>;
    fn notify_or_release(&mut self, swap_data: SwapData) -> Option<i32>;
}

impl<'a, S: SyncPrimitive<'a, CPU>, CPU: CpuVariant> KernelObjectTrait<'a, CPU>
    for KernelObject<'a, S, CPU>
{
    fn remove_thread(&mut self, thread: &'a Thread<'a, CPU>) {
        self.waitqueue.remove(thread);
    }

    fn acquire(
        &mut self,
        thread: &'a Thread<'a, CPU>,
        ticks: u64,
        timeout: Timeout,
    ) -> Option<SwapData> {
        let timeout_instant = timeout.get_ticks().map(|ms| ticks + (ms / MS_PER_TICK));
        let obtained = self.primitive.acquire(thread);
        if obtained.is_none() {
            self.waitqueue.push_back(thread);
            thread.set_pending(self.identifier, timeout_instant);
        }
        obtained.map(|s| s.into()) // Convert S::Swap into SwapData
    }

    fn notify_or_release(&mut self, swap_data: SwapData) -> Option<i32> {
        let unpended_thread = self.waitqueue.pop_head();
        if let Some(thread) = unpended_thread {
            thread.unpend(swap_data);
            Some(0)
        } else {
            let swap = swap_data.try_into().ok().unwrap();
            self.primitive.release(swap);
            Some(0)
        }
    }
}
