#[derive(Copy, Clone)]
pub struct RangeSetting {
    pub value: f32,

    pub min: f32,
    pub max: f32,

    pub title: &'static str,
    pub description: &'static str,
}

impl RangeSetting {
    pub fn new(value: f32, min: f32, max: f32, title: &'static str, description: &'static str) -> Self {
        Self {
            value,

            min,
            max,

            title,
            description,
        }
    }

    pub fn set(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }
}
