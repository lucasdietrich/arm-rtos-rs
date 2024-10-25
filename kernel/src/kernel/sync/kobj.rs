use crate::{
    kernel::{
        kernel::MS_PER_TICK,
        thread::{Thread, Waitqueue},
        timeout::Timeout,
        CpuVariant,
    },
    list::singly_linked as sl,
};

use super::{SwapData, SyncPrimitive};

pub trait KernelObjectTrait<'a, CPU: CpuVariant> {
    /// Remove the thread from the waitqueue
    fn remove_thread(&mut self, thread: &'a Thread<'a, CPU>);

    /// Make the thread try to acquire the primitive or make it pending
    /// for the given duration.
    fn acquire(
        &mut self,
        thread: &'a Thread<'a, CPU>,
        ticks: u64,
        timeout: Timeout,
    ) -> Option<SwapData>;

    /// Notify the first thread in the waitqueue or release the primitive
    /// if no thread is waiting.
    ///
    /// Returns the error code if swap data was properly converted to the expected type.
    /// Otherwise, returns the swap data back to the primitive.
    fn notify_or_release(&mut self, swap_data: SwapData) -> Result<(), SwapData>;
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
}

impl<'a, S: SyncPrimitive<'a, CPU>, CPU: CpuVariant> KernelObjectTrait<'a, CPU>
    for KernelObject<'a, S, CPU>
{
    fn acquire(
        &mut self,
        thread: &'a Thread<'a, CPU>,
        ticks: u64,
        timeout: Timeout,
    ) -> Option<SwapData> {
        let timeout_instant = timeout
            .get_ticks()
            .map(|ms| ticks + (ms as u64 / MS_PER_TICK));
        let obtained = self.primitive.acquire(thread);
        if obtained.is_none() {
            // If the object is not available, make the thread pending on it by
            // appending it at the end of the waitqueue.
            self.waitqueue.push_back(thread);

            // Mark the thread pending until the given instant
            thread.set_pending(self.identifier, timeout_instant);
        }
        obtained.map(|s| s.into()) // Convert S::Swap into SwapData
    }

    fn notify_or_release(&mut self, swap_data: SwapData) -> Result<(), SwapData> {
        let unpended_thread = self.waitqueue.pop_head();
        if let Some(thread) = unpended_thread {
            thread.unpend(swap_data);
        } else {
            // Try convert SwapData into S::Swap, if it fails, return the SwapData back
            let swap = swap_data.try_into()?;

            // Try to release the primitive, if it fails, return the SwapData back
            self.primitive.release(swap).map_err(|s| s.into())?;
        }
        Ok(())
    }

    fn remove_thread(&mut self, thread: &'a Thread<'a, CPU>) {
        self.waitqueue.remove(thread);
    }
}
