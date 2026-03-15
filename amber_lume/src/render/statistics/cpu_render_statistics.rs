use parking_lot::Mutex;
use crate::statistics::measurement::MsMeasurement;
use crate::statistics::statistics::{Smooth, Statistics};

pub struct CpuRenderStatistics {
    internal: Mutex<CpuRenderStatisticsSnapshot>
}

#[derive(Copy, Clone)]
pub struct CpuRenderStatisticsSnapshot {
    pub ui_build: MsMeasurement,
    pub render_commands: MsMeasurement,
}

impl Smooth for CpuRenderStatisticsSnapshot {
    fn smooth(&self, other: &Self, alpha: f32) -> Self {
        Self {
            ui_build: self.ui_build.smoothed(&other.ui_build, alpha),
            render_commands: self.render_commands.smoothed(&other.render_commands, alpha),
        }
    }
}

impl Default for CpuRenderStatistics {
    fn default() -> Self {
        Self {
            internal: Mutex::new(
                CpuRenderStatisticsSnapshot {
                    ui_build: MsMeasurement::new(0.0),
                    render_commands: MsMeasurement::new(0.0),
                }
            )
        }
    }
}

impl CpuRenderStatistics {
    pub fn push(&self, snapshot: CpuRenderStatisticsSnapshot) {
        let mut internal = self.internal.lock();
        *internal = snapshot;
    }
}

impl Statistics for CpuRenderStatistics {
    type Snapshot = CpuRenderStatisticsSnapshot;

    fn snapshot(&self) -> Self::Snapshot {
        let internal = *self.internal.lock();

        Self::Snapshot {
            ..internal
        }
    }
}
