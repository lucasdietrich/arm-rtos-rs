#[derive(Default, Debug)]
pub enum SwapData {
    #[default]
    Empty,
    Signal(u32),
    Ownership,
}

impl Into<i32> for SwapData {
    fn into(self) -> i32 {
        match self {
            SwapData::Empty => 0,
            SwapData::Signal(value) => value as i32,
            SwapData::Ownership => 0,
        }
    }
}
