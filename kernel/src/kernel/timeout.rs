#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timeout {
    Duration(u32),
    #[default]
    Forever,
}

impl Timeout {
    pub fn get_ticks(&self) -> Option<u32> {
        match self {
            Timeout::Duration(ticks) => Some(*ticks),
            Timeout::Forever => None,
        }
    }

    pub fn from_ms(ms: u32) -> Self {
        Timeout::Duration(ms)
    }

    pub fn from_seconds(seconds: u32) -> Self {
        Timeout::Duration(seconds * 1000)
    }

    pub fn is_forever(&self) -> bool {
        matches!(self, Timeout::Forever)
    }

    pub fn is_zero(&self) -> bool {
        matches!(self, Timeout::Duration(0))
    }
}

impl TryFrom<i32> for Timeout {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value == -1 {
            Ok(Timeout::Forever)
        } else if value >= 0 {
            Ok(Timeout::Duration(value as u32))
        } else {
            Err(())
        }
    }
}

impl From<Timeout> for i32 {
    fn from(timeout: Timeout) -> i32 {
        match timeout {
            Timeout::Duration(ticks) => ticks as i32,
            Timeout::Forever => -1,
        }
    }
}