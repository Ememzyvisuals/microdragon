// microdragon-core/src/watch/heartbeat.rs
// Heartbeat system for autonomous daemon

use std::time::{Duration, Instant};

pub struct Heartbeat {
    pub interval_secs: u64,
    pub last_beat: Instant,
    pub beat_count: u64,
}

impl Heartbeat {
    pub fn new(interval_secs: u64) -> Self {
        Self { interval_secs, last_beat: Instant::now(), beat_count: 0 }
    }

    pub fn should_beat(&mut self) -> bool {
        if self.last_beat.elapsed() >= Duration::from_secs(self.interval_secs) {
            self.last_beat = Instant::now();
            self.beat_count += 1;
            true
        } else {
            false
        }
    }
}
