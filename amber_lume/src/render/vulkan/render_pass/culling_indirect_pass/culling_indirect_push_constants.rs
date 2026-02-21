use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct CullingIndirectPushConstants {
    pub culling_views_buffer_device_address: DeviceAddress,

    pub entity_buffer_device_address: DeviceAddress,

    pub submesh_buffer_device_address: DeviceAddress,
    pub model_buffer_device_address: DeviceAddress,

    pub gpu_render_stats_buffer_device_address: DeviceAddress,

    pub culling_views_count: u32,
    pub entity_count: u32,
}

impl CullingIndirectPushConstants {
    pub fn create(
        culling_views_buffer_device_address: DeviceAddress,
        entity_buffer_device_address: DeviceAddress,
        submesh_buffer_device_address: DeviceAddress,
        model_buffer_device_address: DeviceAddress,
        gpu_render_stats_buffer_device_address: DeviceAddress,
        culling_views_count: u32,
        entity_count: u32,
    ) -> Self {
        Self {
            culling_views_buffer_device_address,

            entity_buffer_device_address,

            submesh_buffer_device_address,
            model_buffer_device_address,

            gpu_render_stats_buffer_device_address,

            culling_views_count,
            entity_count,
        }
    }
}
