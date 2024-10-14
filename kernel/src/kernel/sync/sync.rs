use crate::kernel::{thread::Thread, CpuVariant};

use super::traits::SyncPrimitiveTrait;

pub struct Sync;

impl<'a, CPU: CpuVariant> SyncPrimitiveTrait<'a, CPU> for Sync {
    type Notify = ();

    fn sync(&mut self, _thread: Option<&'a Thread<'a, CPU>>, _notify_value: ()) {}

    fn pend(&mut self, _thread: &'a Thread<'a, CPU>) -> Option<()> {
        None
    }
}
