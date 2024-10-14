use crate::{
    kernel::{
        thread::{Thread, Timeout},
        CpuVariant,
    },
    list,
};

use super::{mutex, sem, signal, sync, SyncPrimitiveTrait};

pub enum SyncNotifyValue {
    Sync,
    Signal(u32),
    Semaphore,
    Mutex,
}

pub enum SyncPrimitive<'a, CPU: CpuVariant> {
    Sync(sync::Sync),
    Signal(signal::Signal),
    Semaphore(sem::Semaphore),
    Mutex(mutex::Mutex<'a, CPU>),
}

pub struct KernelObject<'a, S: SyncPrimitiveTrait<'a, CPU>, CPU: CpuVariant> {
    identifier: u32,
    waitqueue: list::List<'a, Thread<'a, CPU>>,
    primitive: S,
}

impl<'a, S: SyncPrimitiveTrait<'a, CPU>, CPU: CpuVariant> KernelObject<'a, S, CPU> {
    pub fn new(identifier: u32, primitive: S) -> Self {
        KernelObject {
            identifier: identifier,
            waitqueue: list::List::empty(),
            primitive,
        }
    }

    pub fn sync(&mut self, notify_value: S::Notify) -> Option<&'a Thread<'a, CPU>> {
        let unpend_thread = self.waitqueue.pop_head();

        let result = self.primitive.sync(unpend_thread, notify_value);

        unpend_thread.map(|thread| thread.unpend(SyncNotifyValue::Sync));

        unpend_thread
    }

    pub fn pend(&mut self, thread: &'a Thread<'a, CPU>, timeout: Timeout) -> Option<S::Notify> {
        let result = self.primitive.pend(thread);
        if result.is_none() {
            self.waitqueue.push_back(thread);
            thread.set_pending(self.identifier, timeout);
        }
        result
    }
}
