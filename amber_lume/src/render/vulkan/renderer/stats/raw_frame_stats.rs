use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::renderer::stats::raw_frame_stat::RawFrameStat;

pub struct RawFrameStats {
    pub gpu_render_time: RawFrameStat,
}

impl RawFrameStats {
    pub fn new(device_context: &DeviceContext) -> Self {
        Self {
            gpu_render_time: RawFrameStat::new(&device_context).expect("Can't register gpu_render_time"),
        }
    }

    pub fn get_gpu_render_time(&self) -> [u64; 2] {
        self.gpu_render_time.pull()
    }

    pub fn destroy(&self, device_context: &DeviceContext) {
        self.gpu_render_time.destroy(&device_context);
    }
}
