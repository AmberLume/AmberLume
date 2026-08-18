#[derive(Copy, Clone)]
pub struct ResidencyLimits {
    pub max_level: u32,
    pub split_factor: f32,

    pub budget: usize,
    pub capacity: usize,
}

impl ResidencyLimits {
    pub const DEFAULT_MAX_LEVEL: u32 = 8;
    pub const DEFAULT_SPLIT_FACTOR: f32 = 2.0;
    pub const DEFAULT_BUDGET: usize = 8;
    pub const DEFAULT_CAPACITY: usize = 1_024;

    pub fn create() -> Self {
        Self {
            max_level: Self::DEFAULT_MAX_LEVEL,
            split_factor: Self::DEFAULT_SPLIT_FACTOR,

            budget: Self::DEFAULT_BUDGET,
            capacity: Self::DEFAULT_CAPACITY,
        }
    }
}
