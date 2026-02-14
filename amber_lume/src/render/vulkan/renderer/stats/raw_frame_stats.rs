use ash::Device;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::renderer::stats::raw_frame_stat::RawFrameStat;

pub struct RawFrameStats {
    pub gpu_render_time: RawFrameStat,
}

impl RawFrameStats {
    pub fn new(
        device: &Device,
        buffer_factory: &ManagedBufferFactory,
    ) -> Self {
        let gpu_render_time = RawFrameStat::new(
            &device,
            buffer_factory,
        ).expect("Can't register gpu_render_time");
        
        Self {
            gpu_render_time,
        }
    }

    pub fn get_gpu_render_time(&self) -> [u64; 2] {
        self.gpu_render_time.pull()
    }

    pub fn destroy(
        self, 
        device: &Device,
        buffer_factory: &ManagedBufferFactory,
    ) {
        self.gpu_render_time.destroy(&device, &buffer_factory);
    }
}
