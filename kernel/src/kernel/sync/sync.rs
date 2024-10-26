use crate::kernel::{thread::Thread, CpuVariant};

use super::traits::{ReleaseOutcome, SyncPrimitive};

#[derive(Default)]
pub struct Sync;

impl<'a, CPU: CpuVariant> SyncPrimitive<'a, CPU> for Sync {
    type Swap = ();

    fn release(&mut self, _released: ()) -> Result<ReleaseOutcome<()>, ()> {
        Ok(ReleaseOutcome::Notified(()))
    }

    fn acquire(&mut self, _thread: &'a Thread<'a, CPU>) -> Option<()> {
        None
    }
}
