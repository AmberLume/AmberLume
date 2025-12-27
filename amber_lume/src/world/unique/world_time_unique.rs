use shipyard::Unique;
use std::time::Instant;

#[derive(Unique, Debug)]
pub struct WorldTimeUnique {
    pub last_instant: Instant,
    pub delta: f32,
    pub elapsed: f32,
    pub scale: f32,
}

impl WorldTimeUnique {
    pub fn new() -> Self {
        Self {
            last_instant: Instant::now(),
            delta: 0.0,
            elapsed: 0.0,
            scale: 1.0,
        }
    }
}
