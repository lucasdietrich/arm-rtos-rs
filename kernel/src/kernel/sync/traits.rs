use crate::kernel::{thread::Thread, CpuVariant};

pub trait SyncPrimitiveTrait<'a, CPU: CpuVariant> {
    // Value passed between threads
    type Notify;

    /// Retrieve first thread from the list and notify the value
    fn sync(&mut self, thread: Option<&'a Thread<'a, CPU>>, notify_value: Self::Notify);

    // Make thread pend on the primitive, return the notified value if available, None otherwise
    fn pend(&mut self, thread: &'a Thread<'a, CPU>) -> Option<Self::Notify>;
}
