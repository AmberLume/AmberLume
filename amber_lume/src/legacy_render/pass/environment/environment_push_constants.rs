use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct EnvironmentPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub camera_buffer_device_address: DeviceAddress,

    _pad0: [u32; 28],
}

impl EnvironmentPushConstants {
    pub fn create(scene_buffer: BufferRange, camera_buffer: BufferRange) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            camera_buffer_device_address: camera_buffer.device_address,

            _pad0: [0; 28],
        }
    }
}
