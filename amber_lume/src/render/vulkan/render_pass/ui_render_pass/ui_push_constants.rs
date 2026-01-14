use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct UiPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
}

impl UiPushConstants {
    pub fn create(
        scene_buffer_device_address: u64,
    ) -> Self {
        Self {
            scene_buffer_device_address,
        }
    }
}
