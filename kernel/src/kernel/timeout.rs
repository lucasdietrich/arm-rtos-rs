use core::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timeout {
    Duration(u64),
    Forever,
}

impl Timeout {
    pub fn get_ticks(&self) -> Option<u64> {
        match self {
            Timeout::Duration(ticks) => Some(*ticks),
            Timeout::Forever => None,
        }
    }

    pub fn advance(&mut self, ticks: u64) {
        *self = match self {
            Timeout::Duration(duration) => Timeout::Duration((*duration).saturating_sub(ticks)),
            Timeout::Forever => Timeout::Forever,
        }
    }

    pub fn is_finite(&self) -> bool {
        matches!(self, Timeout::Duration(..))
    }
}

// impl From<Option<u64>> for Timeout {
//     fn from(value: Option<u64>) -> Self {
//         match value {
//             Some(timeout) => Timeout::Duration(timeout),
//             None => Timeout::Forever,
//         }
//     }
// }

impl Default for Timeout {
    fn default() -> Self {
        Timeout::Forever
    }
}
