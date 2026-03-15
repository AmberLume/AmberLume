use parking_lot::Mutex;
use crate::render::vulkan::device_context::DeviceContext;
use crate::render::statistics::raw::raw_gpu_render_statistics::{RawGpuRenderStatistics, StageMeasurement};
use crate::statistics::measurement::MsMeasurement;
use crate::statistics::statistics::{Smooth, Statistics};

pub struct GpuRenderStatistics {
    internal: Mutex<GpuRenderStatisticsSnapshot>
}

#[derive(Copy, Clone)]
pub struct GpuRenderStatisticsSnapshot {
    pub frame_time: MsMeasurement,

    pub submeshes_culled: u32,
    pub submeshes_rendered: u32,
}

impl Smooth for GpuRenderStatisticsSnapshot {
    fn smooth(&self, other: &Self, alpha: f32) -> Self {
        Self {
            frame_time: self.frame_time.smoothed(&other.frame_time, alpha),

            submeshes_culled: other.submeshes_culled,
            submeshes_rendered: other.submeshes_rendered,
        }
    }
}

impl Default for GpuRenderStatistics {
    fn default() -> Self {
        Self {
            internal: Mutex::new(
                GpuRenderStatisticsSnapshot {
                    frame_time: MsMeasurement::new(0.0),

                    submeshes_culled: 0,
                    submeshes_rendered: 0,
                }
            )
        }
    }
}

impl GpuRenderStatistics {
    pub fn fill(&self, device_context: &DeviceContext, gpu_render_statistics: RawGpuRenderStatistics) {
        let frame_time = Self::ticks_interval_to_ms(&device_context, gpu_render_statistics.render_time);

        let snapshot = GpuRenderStatisticsSnapshot {
            frame_time: MsMeasurement::new(frame_time),

            submeshes_culled: gpu_render_statistics.submeshes_culled,
            submeshes_rendered: gpu_render_statistics.submeshes_rendered,
        };

        let mut internal = self.internal.lock();
        *internal = snapshot;
    }

    fn ticks_interval_to_ms(device_context: &DeviceContext, measurement: StageMeasurement) -> f32 {
        let ticks_delta = measurement.end - measurement.start;

        let nanos_delta = ticks_delta as f64 * device_context.physical_device_info.timestamp_period as f64;

        (nanos_delta / 1_000_000.0) as f32
    }
}

impl Statistics for GpuRenderStatistics {
    type Snapshot = GpuRenderStatisticsSnapshot;

    fn snapshot(&self) -> Self::Snapshot {
        let internal = *self.internal.lock();

        Self::Snapshot {
            ..internal
        }
    }
}
