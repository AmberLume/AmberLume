use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct UiPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub texture_index: u32,
    pub render_mode: u32,
}

impl UiPushConstants {
    pub fn create(
        scene_buffer_device_address: u64,
        texture_index: u32,
        render_mode: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address,
            texture_index,
            render_mode,
        }
    }
}
