#[derive(Default, Debug)]
pub enum SwapData {
    #[default]
    Empty,
    Signal(u32),
    Ownership,
}

impl From<SwapData> for i32 {
    fn from(value: SwapData) -> i32 {
        match value {
            SwapData::Empty => 0,
            SwapData::Signal(value) => value as i32,
            SwapData::Ownership => 0,
        }
    }
}
