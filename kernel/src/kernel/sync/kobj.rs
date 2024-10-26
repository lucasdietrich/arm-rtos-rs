use crate::{
    kernel::{
        thread::{Thread, Waitqueue},
        timeout::TimeoutInstant,
        CpuVariant,
    },
    list::singly_linked as sl,
};

use super::{traits::ReleaseOutcome, SwapData, SyncPrimitive};

pub enum AcquireOutcome {
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
        timeout_instant: TimeoutInstant,
    ) -> AcquireOutcome;

    /// Notify the first thread in the waitqueue or release the primitive
    /// if no thread is waiting.
    ///
    /// Returns the error code if swap data was properly converted to the expected type.
    /// Otherwise, returns the swap data back to the primitive.
    fn release(&mut self, swap_data: SwapData) -> Result<(), SwapData>;

    // TODO: Cancel all thread waiting on the kernel object
    // fn cancel(&mut self);
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
        timeout_instant: TimeoutInstant,
    ) -> AcquireOutcome {
        let obtained = self.primitive.acquire(thread);

        if let Some(swap) = obtained {
            AcquireOutcome::Obtained(swap.into()) // Convert S::Swap into SwapData
        } else if timeout_instant.is_zero() {
            AcquireOutcome::NotObtained
        } else {
            // If the object is not available and a non-zero timeout is given
            // mark the thread as pending and set the timeout

            // appending it at the end of the kobj waitqueue.
            self.waitqueue.push_back(thread);

            // Mark the thread pending until the given instant
            thread.set_pending(self.identifier, timeout_instant);

            AcquireOutcome::Pending
        }
    }

    fn release(&mut self, swap_data: SwapData) -> Result<(), SwapData> {
        // Try convert SwapData into S::Swap, if it fails, return the SwapData back
        let mut swap: S::Swap = swap_data.try_into()?;

        while let Some(unpended_thread) = self.waitqueue.pop_head() {
            unpended_thread.unpend(&swap);
            // Try to notify/release the primitive, if it fails, return the SwapData back
            swap = match self.primitive.release(swap).map_err(|s| s.into())? {
                ReleaseOutcome::Released => break,
                ReleaseOutcome::Notified(swap) => swap,
            }
        }

        Ok(())
    }

    fn remove_thread(&mut self, thread: &'a Thread<'a, CPU>) {
        self.waitqueue.remove(thread);
    }
}
