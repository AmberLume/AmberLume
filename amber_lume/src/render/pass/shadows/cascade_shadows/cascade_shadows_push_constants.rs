use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CascadeShadowsPushConstants {
    pub draw_data_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub shadow_cascades_buffer_device_address: DeviceAddress,

    _pad0: [u32; 26],
}

impl CascadeShadowsPushConstants {
    pub fn create(
        draw_data_buffer: BufferRange,
        entity_buffer: BufferRange,
        shadow_cascades_buffer: BufferRange,
    ) -> Self {
        Self {
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            shadow_cascades_buffer_device_address: shadow_cascades_buffer.device_address,

            _pad0: [0; 26],
        }
    }
}
