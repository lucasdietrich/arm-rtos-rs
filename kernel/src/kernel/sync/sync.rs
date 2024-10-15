use crate::kernel::{thread::Thread, CpuVariant};

use super::traits::SyncPrimitiveTrait;

pub struct Sync;

impl<'a, CPU: CpuVariant> SyncPrimitiveTrait<'a, CPU> for Sync {
    type Swap = ();

    fn sync(&mut self, _notify_value: ()) {}

    fn pend(&mut self, _thread: &'a Thread<'a, CPU>) -> Option<()> {
        None
    }
}
