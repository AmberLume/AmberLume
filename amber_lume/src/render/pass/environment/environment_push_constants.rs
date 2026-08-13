use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct EnvironmentPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
}

impl EnvironmentPushConstants {
    pub fn create(scene_buffer_device_address: DeviceAddress) -> Self {
        Self { scene_buffer_device_address }
    }
}
