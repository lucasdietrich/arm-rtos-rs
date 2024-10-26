use crate::kernel::{thread::Thread, CpuVariant};

use super::{traits::ReleaseOutcome, SwapData, Swappable, SyncPrimitive};

#[derive(Clone, Copy, PartialEq)]
pub struct SignalValue(u32);

impl SignalValue {
    pub const fn new(value: u32) -> Self {
        SignalValue(value)
    }
}

impl From<SignalValue> for SwapData {
    fn from(value: SignalValue) -> SwapData {
        SwapData::Signal(value)
    }
}

impl TryFrom<SwapData> for SignalValue {
    type Error = SwapData;

    fn try_from(swap: SwapData) -> Result<SignalValue, SwapData> {
        match swap {
            SwapData::Signal(value) => Ok(value),
            _ => Err(swap),
        }
    }
}

impl Swappable for SignalValue {
    fn to_syscall_ret(&self) -> i32 {
        self.0 as i32
    }
}

#[derive(Default)]
pub struct Signal {
    value: Option<SignalValue>,
}

impl Signal {
    pub const fn new() -> Self {
        Signal { value: None }
    }
}

impl<'a, CPU: CpuVariant> SyncPrimitive<'a, CPU> for Signal {
    type Swap = SignalValue;

    fn release(
        &mut self,
        notify_value: SignalValue,
    ) -> Result<ReleaseOutcome<SignalValue>, SignalValue> {
        self.value = Some(notify_value);

        Ok(ReleaseOutcome::Notified(notify_value))
    }

    fn acquire(&mut self, _thread: &'a Thread<'a, CPU>) -> Option<SignalValue> {
        self.value
    }
}

impl Signal {
    pub fn reset(&mut self) {
        self.value = None;
    }
}
