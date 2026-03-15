use std::sync::Arc;
use crate::render::statistics::cpu_render_statistics::{CpuRenderStatistics, CpuRenderStatisticsSnapshot};
use crate::render::statistics::gpu_render_statistics::{GpuRenderStatistics, GpuRenderStatisticsSnapshot};
use crate::resources::resource_indices_statistics::{ResourceIndicesStatistics, ResourceIndicesStatisticsSnapshot};
use crate::statistics::statistics::{Smooth, Statistics};

#[derive(Copy, Clone)]
pub struct StatisticsSnapshot {
    pub resource_indices: ResourceIndicesStatisticsSnapshot,

    pub cpu_render: CpuRenderStatisticsSnapshot,
    pub gpu_render: GpuRenderStatisticsSnapshot,
}

impl StatisticsSnapshot {
    pub fn smoothed(&self, other: &Self, alpha: f32) -> Self {
        Self {
            resource_indices: self.resource_indices.smooth(&other.resource_indices, alpha),

            cpu_render: self.cpu_render.smooth(&other.cpu_render, alpha),
            gpu_render: self.gpu_render.smooth(&other.gpu_render, alpha),
        }
    }
}

pub struct StatisticsContext {
    pub resource_indices: Arc<ResourceIndicesStatistics>,

    pub cpu_render: Arc<CpuRenderStatistics>,
    pub gpu_render: Arc<GpuRenderStatistics>,
}

impl Default for StatisticsContext {
    fn default() -> Self {
        Self {
            resource_indices: Arc::new(ResourceIndicesStatistics::default()),

            cpu_render: Arc::new(CpuRenderStatistics::default()),
            gpu_render: Arc::new(GpuRenderStatistics::default()),
        }
    }
}

impl StatisticsContext {
    pub fn snapshot(&self) -> StatisticsSnapshot {
        StatisticsSnapshot {
            resource_indices: self.resource_indices.snapshot(),

            cpu_render: self.cpu_render.snapshot(),
            gpu_render: self.gpu_render.snapshot(),
        }
    }
}
