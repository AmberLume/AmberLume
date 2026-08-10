use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShProjectPushConstants {
    pub scene_buffer_device_address: DeviceAddress,

    _pad0: [u32; 30],
}

impl ShProjectPushConstants {
    pub fn create(scene_buffer_device_address: DeviceAddress) -> Self {
        Self {
            scene_buffer_device_address,

            _pad0: [0; 30],
        }
    }
}
