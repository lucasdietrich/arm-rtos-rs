//! Provides traits and implementations for synchronization primitives.

use super::SwapData;
use crate::kernel::{thread::Thread, CpuVariant};

/// A trait for types that can be passed between threads during synchronization.
///
/// Any type that implements `Into<SwapData>` automatically implements `Swappable`.
pub trait Swappable: Into<SwapData> + TryFrom<SwapData, Error = SwapData> {
    fn to_syscall_ret(&self) -> i32;
}

/// Implements `Into<SwapData>` for the unit type `()`.
///
/// This allows the unit type to be used in synchronization primitives where no actual data needs to be passed.
impl From<()> for SwapData {
    fn from(_: ()) -> SwapData {
        SwapData::Empty
    }
}

impl TryFrom<SwapData> for () {
    type Error = SwapData;

    fn try_from(swap: SwapData) -> Result<Self, SwapData> {
        match swap {
            SwapData::Empty => Ok(()),
            _ => Err(swap),
        }
    }
}

impl Swappable for () {
    fn to_syscall_ret(&self) -> i32 {
        0
    }
}

pub enum ReleaseOutcome<S: Swappable> {
    /// The primitive has been released
    Released,
    /// The primitive has been notified to the first thread in the waitqueue
    /// but it is still available for other threads (e.g. signaling)
    Notified(S),
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

    // type Init: Default + Clone + Copy;

    // fn init(&self) -> Self::Init {
    //     Default::default()
    // }

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

    /// Releases the synchronization primitive with the given swap value as
    /// no thread was waiting for the primitive.
    ///
    /// Typically called when a thread has finished using the resource protected
    /// by the synchronization primitive and no thread is waiting to acquire it.
    /// So the swap value is given back to the primitive.
    ///
    /// # Parameters
    ///
    /// - `released`: The value to pass to the next waiting thread.
    ///
    /// # Returns
    ///
    /// - `Ok(ReleaseOutcome::Released)` if the primitive was released successfully.
    /// - `Ok(ReleaseOutcome::Notified(Self::Swap))` if the primitive was notified successfully
    ///   and the swap value is still available for other threads.
    /// - `Err(Self::Swap)` if the primitive was not released successfully.
    fn release(&mut self, released: Self::Swap) -> Result<ReleaseOutcome<Self::Swap>, Self::Swap>;

    // fn cancel(&mut self) {
    //     todo!()
    // }
}
