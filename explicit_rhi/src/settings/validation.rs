#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Validation {
    pub api: bool,
    pub synchronization: bool,
    pub best_practices: bool,
    pub gpu_assisted: bool,
}

impl Validation {
    pub const OFF: Self = Self {
        api: false,
        synchronization: false,
        best_practices: false,
        gpu_assisted: false,
    };

    pub fn is_enabled(&self) -> bool {
        self.api || self.synchronization || self.best_practices || self.gpu_assisted
    }
}
