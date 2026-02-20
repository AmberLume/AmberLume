use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CullingPushConstants {
    pub scene_buffer_device_address: DeviceAddress,

    pub gpu_render_stats_buffer_device_address: DeviceAddress,
    
    pub entity_count: u32,
    _pad0: u32,
}

impl CullingPushConstants {
    pub fn create(
        scene_buffer_device_address: DeviceAddress,
        gpu_render_stats_buffer_device_address: DeviceAddress,
        entity_count: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address,

            gpu_render_stats_buffer_device_address,
            
            entity_count,
            _pad0: 0,
        }
    }
}
