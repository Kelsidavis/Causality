// Time and delta time tracking

use std::time::Instant;

pub struct TimeState {
    start: Instant,
    last_frame: Instant,
    delta: f32,
}

impl TimeState {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            last_frame: now,
            delta: 0.0,
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        self.delta = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }

    pub fn elapsed(&self) -> f32 {
        (Instant::now() - self.start).as_secs_f32()
    }
}
