use crate::render::pass::passes_statistics::PassesStatistics;
use crate::statistics::time_measurement::TimeMeasurement;

pub struct RenderStatisticsMeasurement {
    pub total_time: TimeMeasurement,
    pub collect_record_commands: TimeMeasurement,
}

impl RenderStatisticsMeasurement {
    pub fn new() -> Self {
        Self {
            total_time: TimeMeasurement::new(),
            collect_record_commands: TimeMeasurement::new(),
        }
    }
}

pub struct RenderStatistics {
    pub total_time: u64,
    pub collect_record_commands: u64,
    
    pub passes_statistics: PassesStatistics,
}
