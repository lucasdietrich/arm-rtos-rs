use crate::kernel::{thread::Thread, CpuVariant};

use super::traits::SyncPrimitive;

pub struct Sync;

impl<'a, CPU: CpuVariant> SyncPrimitive<'a, CPU> for Sync {
    type Swap = ();

    fn release(&mut self, _released: ()) -> Result<(), ()> {
        Ok(())
    }

    fn acquire(&mut self, _thread: &'a Thread<'a, CPU>) -> Option<()> {
        None
    }
}
