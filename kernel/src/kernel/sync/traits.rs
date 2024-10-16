//! Provides traits and implementations for synchronization primitives.

use super::SwapData;
use crate::kernel::{thread::Thread, CpuVariant};

/// A trait for types that can be passed between threads during synchronization.
///
/// Any type that implements `Into<SwapData>` automatically implements `Swappable`.
pub trait Swappable: Into<SwapData> {}

/// Automatically implements `Swappable` for any type that implements `Into<SwapData>`.
impl<T> Swappable for T where T: Into<SwapData> {}

/// Implements `Into<SwapData>` for the unit type `()`.
///
/// This allows the unit type to be used in synchronization primitives where no actual data needs to be passed.
impl Into<SwapData> for () {
    fn into(self) -> SwapData {
        SwapData::Empty
    }
}

/// A synchronization primitive that allows threads to synchronize access to shared resources.
///
/// Implementations of this trait define how threads can synchronize using swap values,
/// enabling the creation of synchronization mechanisms like mutexes, semaphores, etc.
pub trait SyncPrimitive<'a, CPU: CpuVariant> {
    /// The type of value that is passed between threads during synchronization.
    ///
    /// This type must implement the `Swappable` trait.
    type Swap: Swappable;

    /// Attempts to acquire the synchronization primitive for the given thread.
    ///
    /// If the primitive is available, the thread acquires it and may receive a swap value.
    /// If the primitive is not available, the thread is made to wait until it can be notified.
    ///
    /// # Parameters
    ///
    /// - `thread`: A reference to the thread attempting to acquire the synchronization primitive.
    ///
    /// # Returns
    ///
    /// - `Some(Self::Swap)` if the primitive was acquired and a swap value is available.
    /// - `None` if the thread has been made to wait and no value is immediately available.
    fn acquire(&mut self, thread: &'a Thread<'a, CPU>) -> Option<Self::Swap>;

    /// Releases the synchronization primitive and notifies the next waiting thread with the provided value.
    ///
    /// Typically called when a thread has finished using the resource protected by the synchronization primitive
    /// and wants to wake up another waiting thread.
    ///
    /// # Parameters
    ///
    /// - `released`: The value to pass to the next waiting thread.
    fn release(&mut self, released: Self::Swap);
}
