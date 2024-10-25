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

pub enum AcquireResult {
    /// The thread has obtained the primitive
    Obtained(SwapData),
    /// The thread has not obtained the primitive
    /// and the timeout is zero
    NotObtained,
    /// The thread has been marked as pending
    Pending,
}

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
    ) -> AcquireResult;

    /// Notify the first thread in the waitqueue or release the primitive
    /// if no thread is waiting.
    ///
    /// Returns the error code if swap data was properly converted to the expected type.
    /// Otherwise, returns the swap data back to the primitive.
    fn notify_or_release(&mut self, swap_data: SwapData) -> Result<(), SwapData>;
}

/// Implement a concrete synchronization primitive
pub struct KernelObject<'a, S: SyncPrimitive<'a, CPU>, CPU: CpuVariant> {
    /// Identifier of the kernel object (currently index of the object in the kernel object table)
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
            identifier,
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
    ) -> AcquireResult {
        let obtained = self.primitive.acquire(thread);

        if let Some(swap) = obtained {
            AcquireResult::Obtained(swap.into()) // Convert S::Swap into SwapData
        } else if timeout.is_zero() {
            AcquireResult::NotObtained
        } else {
            // If the object is not available and a non-zero timeout is given
            // mark the thread as pending and set the timeout

            // appending it at the end of the kobj waitqueue.
            self.waitqueue.push_back(thread);

            // Calculate the instant when the thread should be woken up
            let timeout_instant = timeout
                .get_ticks()
                .map(|ms| ticks + (ms as u64 / MS_PER_TICK));

            // Mark the thread pending until the given instant
            thread.set_pending(self.identifier, timeout_instant);

            AcquireResult::Pending
        }
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
