#[repr(u32)]
pub enum IntervalMeasurement {
    Start = 0,
    End = 1,

    Count = 2,
}

#[repr(C)]
#[derive(Clone)]
pub struct IntervalMeasurementResult {
    pub start: u64,
    pub end: u64,
}
