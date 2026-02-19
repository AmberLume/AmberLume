use crate::render::vulkan::renderer::stats::frame_stats::FrameStats;

#[derive(Debug, Copy, Clone)]
pub struct SystemStats {
    pub world_iteration_time: Option<f32>,
    pub entities_ecs: u32,
    pub submesh_rendered: u32,
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
                world_iteration_time: None,
                entities_ecs: 0,
                submesh_rendered: 0,
                last_frame_stats: None,
            },
        }
    }

    pub fn register_world_iteration_time(&mut self, time: f32) {
        self.in_progress.world_iteration_time = Some(time);
    }

    pub fn register_ecs_entities_count(&mut self, count: u32) {
        self.in_progress.entities_ecs = count;
    }

    pub fn register_submesh_rendered(&mut self, count: u32) {
        self.in_progress.submesh_rendered = count;
    }

    pub fn register_frame_stats(&mut self, frame_stats: FrameStats) {
        self.in_progress.last_frame_stats = Some(frame_stats);
    }

    pub fn publish(&mut self) {
        self.current = Some(self.in_progress);

        self.in_progress = SystemStats {
            world_iteration_time: None,
            entities_ecs: 0,
            submesh_rendered: 0,
            last_frame_stats: None,
        };
    }

    pub fn get_snapshot(&self) -> &Option<SystemStats> {
        &self.current
    }
}
