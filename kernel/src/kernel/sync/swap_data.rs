use super::{signal::SignalValue, Swappable};

#[derive(Default)]
pub enum SwapData {
    #[default]
    Empty,
    Signal(SignalValue),
    Ownership,
}

impl SwapData {
    pub fn to_syscall_ret(&self) -> i32 {
        match self {
            SwapData::Empty => 0,
            SwapData::Signal(value) => value.to_syscall_ret(),
            SwapData::Ownership => 0,
        }
    }
}
