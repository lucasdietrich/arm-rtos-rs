#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timeout {
    // Duration in milliseconds
    Duration(u32),
    #[default]
    Forever,
}

impl Timeout {
    pub fn get_ms(&self) -> Option<u32> {
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

impl From<u32> for Timeout {
    fn from(value: u32) -> Self {
        if value == u32::MAX {
            Timeout::Forever
        } else {
            Timeout::Duration(value)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutInstant {
    At(u64), // Instant (agnostic to the clock source)
    Never,
}

impl TimeoutInstant {
    pub fn new_at(instant: u64) -> Self {
        TimeoutInstant::At(instant)
    }

    pub fn new_never() -> Self {
        TimeoutInstant::Never
    }

    pub fn is_never(&self) -> bool {
        matches!(self, TimeoutInstant::Never)
    }

    pub fn is_zero(&self) -> bool {
        matches!(self, TimeoutInstant::At(0))
    }

    pub fn is_past(&self, now: u64) -> bool {
        match self {
            TimeoutInstant::At(at) => *at <= now,
            TimeoutInstant::Never => false,
        }
    }
}
