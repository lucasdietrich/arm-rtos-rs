#[derive(Default, Debug)]
pub enum SwapData {
    #[default]
    Empty,
    Signal(u32),
    Ownership,
}

impl SwapData {
    pub fn to_syscall_ret(&self) -> i32 {
        match self {
            SwapData::Empty => 0,
            SwapData::Signal(value) => *value as i32,
            SwapData::Ownership => 0,
        }
    }
}

impl From<SwapData> for i32 {
    fn from(value: SwapData) -> i32 {
        value.to_syscall_ret()
    }
}
