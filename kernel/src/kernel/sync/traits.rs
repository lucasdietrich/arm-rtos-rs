use crate::kernel::{thread::Thread, CpuVariant};

use super::SwapData;

pub trait Swappable: Into<SwapData> {}

// Automatically implement Swappable for types which
// implement Into<SwapData>
impl<T> Swappable for T where T: Into<SwapData> {}

impl Into<SwapData> for () {
    fn into(self) -> SwapData {
        SwapData::Empty
    }
}

pub trait SyncPrimitiveTrait<'a, CPU: CpuVariant> {
    // Value passed (swapped) between threads
    type Swap: Swappable;

    /// Retrieve first thread from the list and notify the value
    fn sync(&mut self, notify_value: Self::Swap);

    // Make thread pend on the primitive, return the notified value if available, None otherwise
    fn pend(&mut self, thread: &'a Thread<'a, CPU>) -> Option<Self::Swap>;
}
