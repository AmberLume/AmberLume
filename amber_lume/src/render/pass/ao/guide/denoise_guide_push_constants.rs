use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct DenoiseGuidePushConstants {
    pub scene_buffer_device_address: DeviceAddress,

    pub depth_descriptor_id: u32,
    pub normal_descriptor_id: u32,
    pub guide_storage_id: u32,
    pub width: u32,
    pub height: u32,

    _pad0: [u32; 1],
}

impl DenoiseGuidePushConstants {
    pub fn create(
        scene_buffer: BufferRange,
        depth_descriptor_id: u32,
        normal_descriptor_id: u32,
        guide_storage_id: u32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,

            depth_descriptor_id,
            normal_descriptor_id,
            guide_storage_id,
            width,
            height,

            _pad0: [0; 1],
        }
    }
}
