use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct DebugLayerPushConstants {
    pub camera_buffer_device_address: DeviceAddress,

    pub texture_index: u32,
    pub layer_kind: u32,
    pub shadow_colored: u32,

    pub denoise_history: f32,

    _pad0: [u32; 26],
}

impl DebugLayerPushConstants {
    pub fn create(
        camera_buffer: BufferRange,
        texture_index: u32,
        layer_kind: u32,
        shadow_colored: u32,
        denoise_history: f32,
    ) -> Self {
        Self {
            camera_buffer_device_address: camera_buffer.device_address,

            texture_index,
            layer_kind,
            shadow_colored,

            denoise_history,

            _pad0: [0; 26],
        }
    }
}
