use std::time::{Duration, Instant};

pub struct Benchmark {
    ring: Vec<Duration>,
    reading: usize,
}

impl Benchmark {
    pub fn new(samples: usize) -> Self {
        Self {
            ring: vec![Duration::from_secs(0); samples],
            reading: 0,
        }
    }

    pub fn with_benchmark<F>(&mut self, mut f: F) -> Duration
    where
        F: FnMut(),
    {
        let timer = Instant::now();
        f();
        let time = timer.elapsed();

        self.reading = (self.reading + 1) % self.ring.capacity();
        self.ring[self.reading] = time;

        time
    }

    pub fn last_time(&self) -> Time {
        let time = self.ring[self.reading].as_micros();
        if time <= 1_000 {
            Time::Microsecond(time as u32)
        } else if time <= 1_000_000 {
            Time::Millisecond((time / 1_000) as u32)
        } else {
            Time::Second((time / 1_000_000) as u32)
        }
    }

    pub fn average_time(&self) -> Time {
        let mut total = 0;
        for time in &self.ring {
            total += time.as_micros();
        }

        let time = total / self.ring.len() as u128;
        if time <= 1_000 {
            Time::Microsecond(time as u32)
        } else if time <= 1_000_000 {
            Time::Millisecond((time / 1_000) as u32)
        } else {
            Time::Second((time / 1_000_000) as u32)
        }
    }

    pub fn max_time(&self) -> Time {
        let mut max = Duration::from_secs(0);
        for time in &self.ring {
            if *time > max {
                max = *time;
            }
        }

        let time = max.as_micros();
        if time <= 1_000 {
            Time::Microsecond(time as u32)
        } else if time <= 1_000_000 {
            Time::Millisecond((time / 1_000) as u32)
        } else {
            Time::Second((time / 1_000_000) as u32)
        }
    }
}

pub enum Time {
    Second(u32),
    Millisecond(u32),
    Microsecond(u32),
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Time::Second(time) => write!(f, "{:>4}s ", time),
            Time::Millisecond(time) => write!(f, "{:>4}ms", time),
            Time::Microsecond(time) => write!(f, "{:>4}µs", time),
        }
    }
}
