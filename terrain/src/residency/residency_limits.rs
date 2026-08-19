#[derive(Copy, Clone)]
pub struct ResidencyLimits {
    pub max_level: u32,
    pub split_factor: f32,

    pub budget: usize,
    pub capacity: usize,

    pub ray_tracing_distance: f32,
    pub hysteresis: f32,
    pub retire_delay: u32,
}

impl ResidencyLimits {
    pub const DEFAULT_MAX_LEVEL: u32 = 6;
    pub const DEFAULT_SPLIT_FACTOR: f32 = 2.0;
    pub const DEFAULT_BUDGET: usize = 8;
    pub const DEFAULT_CAPACITY: usize = 1_024;
    pub const DEFAULT_RAY_TRACING_DISTANCE: f32 = 384.0;
    pub const DEFAULT_HYSTERESIS: f32 = 1.25;
    pub const DEFAULT_RETIRE_DELAY: u32 = 60;

    pub fn create() -> Self {
        Self {
            max_level: Self::DEFAULT_MAX_LEVEL,
            split_factor: Self::DEFAULT_SPLIT_FACTOR,

            budget: Self::DEFAULT_BUDGET,
            capacity: Self::DEFAULT_CAPACITY,

            ray_tracing_distance: Self::DEFAULT_RAY_TRACING_DISTANCE,
            hysteresis: Self::DEFAULT_HYSTERESIS,
            retire_delay: Self::DEFAULT_RETIRE_DELAY,
        }
    }
}
