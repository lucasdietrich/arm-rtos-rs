//! Provides a mutual exclusion primitive (`Mutex`) for synchronizing access to shared data
//! between threads in a kernel environment.

use crate::kernel::{thread::Thread, CpuVariant};

use super::{SwapData, SyncPrimitive};

/// A token representing the ownership of a mutex.
///
/// This struct is used internally to indicate that a thread has successfully acquired
/// ownership of a `Mutex`. It serves as a marker in synchronization operations.
pub struct Ownership;

impl Into<SwapData> for Ownership {
    fn into(self) -> SwapData {
        SwapData::Ownership
    }
}

impl TryFrom<SwapData> for Ownership {
    type Error = SwapData;

    fn try_from(swap: SwapData) -> Result<Self, SwapData> {
        match swap {
            SwapData::Ownership => Ok(Ownership),
            _ => Err(swap),
        }
    }
}

/// A mutual exclusion primitive useful for protecting shared data.
///
/// The `Mutex` structure keeps track of the owning thread and ensures that only one
/// thread can access the protected data at a time. It implements the `SyncPrimitive` trait
/// to integrate with the kernel's synchronization mechanisms.
pub struct Mutex<'a, CPU: CpuVariant> {
    owner: Option<&'a Thread<'a, CPU>>,
}

impl<'a, CPU: CpuVariant> Mutex<'a, CPU> {
    /// Creates a new `Mutex` with no owner.
    pub const fn new() -> Self {
        Mutex { owner: None }
    }
}

impl<'a, CPU: CpuVariant> SyncPrimitive<'a, CPU> for Mutex<'a, CPU> {
    type Swap = Ownership;

    /// Releases the mutex, making it available for other threads to acquire.
    ///
    /// This method is called when a thread releases the mutex ownership.
    ///
    /// # Arguments
    ///
    /// * `_released` - The ownership token to be consumed upon release.
    fn release(&mut self, _released: Ownership) -> Result<(), Ownership> {
        if self.owner.is_none() {
            Err(Ownership)
        } else {
            self.owner = None;
            Ok(())
        }
    }

    /// Attempts to acquire the mutex for the given thread.
    ///
    /// If the mutex is not currently owned, the thread acquires it and receives an
    /// ownership token. If the mutex is already owned, `None` is returned, indicating
    /// that the thread must wait or retry.
    ///
    /// # Arguments
    ///
    /// * `thread` - A reference to the thread attempting to acquire the mutex.
    ///
    /// # Returns
    ///
    /// * `Some(Ownership)` if the mutex was successfully acquired.
    /// * `None` if the mutex is already owned by another thread.
    fn acquire(&mut self, thread: &'a Thread<'a, CPU>) -> Option<Ownership> {
        if self.owner.is_none() {
            self.owner = Some(thread);
            Some(Ownership)
        } else {
            None
        }
        // TODO, what to do if the mutex is already owned by the same thread?
    }
}
