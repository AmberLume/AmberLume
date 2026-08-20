#[derive(Copy, Clone)]
pub struct ResidencyLimits {
    pub max_level: u32,
    pub split_factor: f32,

    pub capacity: usize,
    pub rebuild_margin: f32,

    pub ray_tracing_distance: f32,
}

impl ResidencyLimits {
    pub const DEFAULT_MAX_LEVEL: u32 = 6;
    pub const DEFAULT_SPLIT_FACTOR: f32 = 2.0;
    pub const DEFAULT_CAPACITY: usize = 1_024;
    pub const DEFAULT_REBUILD_MARGIN: f32 = 48.0;
    pub const DEFAULT_RAY_TRACING_DISTANCE: f32 = 384.0;

    pub fn create() -> Self {
        Self {
            max_level: Self::DEFAULT_MAX_LEVEL,
            split_factor: Self::DEFAULT_SPLIT_FACTOR,

            capacity: Self::DEFAULT_CAPACITY,
            rebuild_margin: Self::DEFAULT_REBUILD_MARGIN,

            ray_tracing_distance: Self::DEFAULT_RAY_TRACING_DISTANCE,
        }
    }
}
