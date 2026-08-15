use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GtaoDepthPushConstants {
    pub scene_buffer_device_address: DeviceAddress,

    pub depth_descriptor_id: u32,
    pub view_z_storage_id: u32,
    pub width: u32,
    pub height: u32,

    _pad0: [u32; 26],
}

impl GtaoDepthPushConstants {
    pub fn create(
        scene_buffer_device_address: DeviceAddress,
        depth_descriptor_id: u32,
        view_z_storage_id: u32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address,

            depth_descriptor_id,
            view_z_storage_id,
            width,
            height,

            _pad0: [0; 26],
        }
    }
}
