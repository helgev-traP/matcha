use std::time::{Duration, Instant};

pub struct Ticker {
    start_time: Instant,
    last_frame_time: Instant,
    current_time: Duration,
}

impl Ticker {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame_time: now,
            current_time: Duration::from_secs(0),
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        self.current_time = now.duration_since(self.start_time);
        self.last_frame_time = now;
    }

    pub fn current_time(&self) -> Duration {
        self.current_time
    }
}

impl Default for Ticker {
    fn default() -> Self {
        Self::new()
    }
}
