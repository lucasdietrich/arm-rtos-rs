pub trait SchedulerTimer {
    // Schedule the timer to expire in the given duration (in us)
    // Expiration results in an interrupt allowing kernel to handle it
    fn sched(&mut self, us: u32);

    // Return the remaining time until expiration
    fn get_remaining_us(&self) -> Option<u32>;
}
