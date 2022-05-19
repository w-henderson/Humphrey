use std::time::Duration;

pub struct Heartbeat {
    pub(crate) interval: Duration,
    pub(crate) timeout: Duration,
}

impl Heartbeat {
    pub fn new(interval: Duration, timeout: Duration) -> Self {
        Self { interval, timeout }
    }
}

impl Default for Heartbeat {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(5),
            timeout: Duration::from_secs(10),
        }
    }
}
