//! Module providing kernel objects and synchronization primitives.
//!
//! This module defines the `KernelObject` struct and associated traits for managing synchronization
//! primitives like mutexes, semaphores, etc., in a kernel environment. It includes mechanisms for
//! acquiring and releasing these primitives, as well as managing threads that are waiting on them.

use crate::{
    kernel::{
        thread::{Thread, Waitqueue},
        timeout::TimeoutInstant,
        CpuVariant,
    },
    list::singly_linked as sl,
};

use super::{traits::ReleaseOutcome, SwapData, SyncPrimitive};

/// The outcome of attempting to acquire a synchronization primitive.
///
/// This enum represents the possible results when a thread tries to acquire a kernel object
/// (e.g., mutex, semaphore).
pub enum AcquireOutcome {
    /// The thread has successfully obtained the primitive.
    Obtained(SwapData),
    /// The thread did not obtain the primitive, and the timeout is zero.
    NotObtained,
    /// The thread has been marked as pending (waiting) due to a non-zero timeout.
    Pending,
}

/// Trait defining the behavior of kernel objects (synchronization primitives).
///
/// This trait provides methods for acquiring and releasing synchronization primitives,
/// as well as removing threads from the waitqueue.
pub trait KernelObjectTrait<'a, CPU: CpuVariant> {
    /// Removes a thread from the waitqueue of the kernel object.
    ///
    /// # Arguments
    ///
    /// * `thread` - A reference to the thread to be removed from the waitqueue.
    fn remove_thread(&mut self, thread: &'a Thread<'a, CPU>);

    /// Attempts to acquire the synchronization primitive for the given thread.
    ///
    /// If the primitive cannot be immediately acquired, and a non-zero timeout is specified,
    /// the thread is added to the waitqueue and marked as pending.
    ///
    /// # Arguments
    ///
    /// * `thread` - The thread attempting to acquire the primitive.
    /// * `timeout_instant` - The timeout after which the thread should stop waiting.
    ///
    /// # Returns
    ///
    /// An `AcquireOutcome` indicating whether the primitive was obtained, not obtained,
    /// or if the thread is pending.
    fn acquire(
        &mut self,
        thread: &'a Thread<'a, CPU>,
        timeout_instant: TimeoutInstant,
    ) -> AcquireOutcome;

    /// Releases the synchronization primitive and notifies waiting threads.
    ///
    /// This method releases the primitive, possibly allowing other threads to acquire it.
    /// It may also notify threads that are waiting in the waitqueue.
    ///
    /// # Arguments
    ///
    /// * `swap_data` - The data to released to the primitive.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the release was successful.
    /// * `Err(SwapData)` if the provided `swap_data` could not be used, returning it back.
    fn release(&mut self, swap_data: SwapData) -> Result<(), SwapData>;

    // TODO: Cancel all threads waiting on the kernel object.
    // fn cancel(&mut self);
}

/// A concrete implementation of a kernel object (synchronization primitive).
///
/// This struct wraps a synchronization primitive and manages the threads waiting on it.
///
/// # Type Parameters
///
/// * `'a` - The lifetime associated with the kernel object.
/// * `S` - The type of the synchronization primitive implementing `SyncPrimitive`.
/// * `CPU` - The CPU variant implementing `CpuVariant`.
pub struct KernelObject<'a, S: SyncPrimitive<'a, CPU>, CPU: CpuVariant> {
    /// Identifier of the kernel object (currently the index in the kernel object table).
    identifier: u32,
    /// List of threads waiting on the kernel object.
    waitqueue: sl::List<'a, Thread<'a, CPU>, Waitqueue>,
    /// The synchronization primitive implementation.
    primitive: S,
}

impl<'a, S: SyncPrimitive<'a, CPU>, CPU: CpuVariant> KernelObject<'a, S, CPU> {
    /// Creates a new kernel object with the given synchronization primitive.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The unique identifier for this kernel object.
    /// * `primitive` - The synchronization primitive to be managed.
    ///
    /// # Returns
    ///
    /// A new instance of `KernelObject`.
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
    /// Attempts to acquire the synchronization primitive for the given thread.
    ///
    /// If the primitive is available, it is acquired, and `Obtained` is returned.
    /// If not, and the timeout is zero, `NotObtained` is returned.
    /// If the timeout is non-zero, the thread is added to the waitqueue and marked as pending.
    ///
    /// # Arguments
    ///
    /// * `thread` - The thread attempting to acquire the primitive.
    /// * `timeout_instant` - The timeout instant after which the thread should stop waiting.
    ///
    /// # Returns
    ///
    /// An `AcquireOutcome` indicating the result of the acquisition attempt.
    fn acquire(
        &mut self,
        thread: &'a Thread<'a, CPU>,
        timeout_instant: TimeoutInstant,
    ) -> AcquireOutcome {
        let obtained = self.primitive.acquire(thread);

        if let Some(swap) = obtained {
            // The primitive was successfully acquired.
            AcquireOutcome::Obtained(swap.into()) // Convert S::Swap into SwapData.
        } else if timeout_instant.is_zero() {
            // The primitive is not available and the timeout is zero.
            AcquireOutcome::NotObtained
        } else {
            // The primitive is not available and a non-zero timeout is specified.
            // Mark the thread as pending and set the timeout.

            // Append the thread to the end of the kernel object's waitqueue.
            self.waitqueue.push_back(thread);

            // Mark the thread as pending until the specified timeout instant.
            thread.set_pending(self.identifier, timeout_instant);

            AcquireOutcome::Pending
        }
    }

    /// Releases the synchronization primitive and notifies waiting threads.
    ///
    /// This method attempts to release the primitive, and if there are threads waiting in the
    /// waitqueue, it notifies them according to the primitive's release logic.
    ///
    /// # Arguments
    ///
    /// * `swap_data` - The data to be released to the primitive.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the release was successful.
    /// * `Err(SwapData)` if the provided `swap_data` could not be converted to the expected type,
    ///   returning it back.
    fn release(&mut self, swap_data: SwapData) -> Result<(), SwapData> {
        // Try to convert SwapData into the primitive's expected swap type.
        let mut swap: S::Swap = swap_data.try_into()?;

        while let Some(unpended_thread) = self.waitqueue.pop_head() {
            // Unpend the thread with the provided swap data.
            unpended_thread.unpend(&swap);

            // Try to release or notify the primitive.
            swap = match self.primitive.release(swap).map_err(|s| s.into())? {
                ReleaseOutcome::Released => break, // The primitive has been released.
                ReleaseOutcome::Notified(swap) => swap, // Continue notifying next thread.
            }
        }

        Ok(())
    }

    /// Removes a thread from the waitqueue of the kernel object.
    ///
    /// If the thread is waiting on this kernel object, it will be removed from the waitqueue.
    ///
    /// # Arguments
    ///
    /// * `thread` - The thread to be removed from the waitqueue.
    fn remove_thread(&mut self, thread: &'a Thread<'a, CPU>) {
        self.waitqueue.remove(thread);
    }
}
