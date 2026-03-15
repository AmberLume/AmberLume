use std::time::Instant;

#[derive(Copy, Clone)]
pub struct MsMeasurement {
    pub value: f32,
}

impl MsMeasurement {
    pub fn new(value: f32) -> Self {
        Self { value }
    }

    pub fn smoothed(&self, other: &Self, alpha: f32) -> Self {
        Self {
            value: self.value + (other.value - self.value) * alpha,
        }
    }
}

pub struct MeasurementInstant {
    instant: Instant,
}

impl MeasurementInstant {
    pub fn start() -> Self {
        Self {
            instant: Instant::now(),
        }
    }

    pub fn capture(self) -> MsMeasurement {
        let instant = Instant::now();

        MsMeasurement::new(instant.duration_since(self.instant).as_secs_f32() * 1000.0)
    }
}
