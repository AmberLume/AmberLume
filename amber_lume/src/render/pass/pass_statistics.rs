use crate::statistics::time_measurement::TimeMeasurement;

pub struct PassStatistics {
    pub prepare: u64,
    pub collect_render_commands: u64,
}

pub struct PassStatisticsMeasurement {
    pub prepare: TimeMeasurement,
    pub collect_render_commands: TimeMeasurement,
}

impl PassStatisticsMeasurement {
    pub fn new() -> Self {
        Self {
            prepare: TimeMeasurement::new(),
            collect_render_commands: TimeMeasurement::new(),
        }
    }
    
    pub fn collect(&self) -> PassStatistics {
        PassStatistics {
            prepare: self.prepare.collect(),
            collect_render_commands: self.collect_render_commands.collect(),
        }
    }
}
