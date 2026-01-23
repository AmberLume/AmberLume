use crate::render::vulkan::renderer::stats::frame_stats::FrameStats;

#[derive(Debug, Copy, Clone)]
pub struct SystemStats {
    pub last_frame_stats: Option<FrameStats>,
}

pub struct SystemStatsHolder {
    current: Option<SystemStats>,
    in_progress: SystemStats,
}

impl SystemStatsHolder {
    pub fn create() -> Self {
        Self {
            current: None,
            in_progress: SystemStats {
                last_frame_stats: None,
            },
        }
    }

    pub fn register_frame_stats(&mut self, frame_stats: FrameStats) {
        self.in_progress.last_frame_stats = Some(frame_stats);
    }

    pub fn publish(&mut self) {
        self.current = Some(self.in_progress);

        self.in_progress = SystemStats {
            last_frame_stats: None,
        };
    }

    pub fn get_snapshot(&self) -> &Option<SystemStats> {
        &self.current
    }
}
