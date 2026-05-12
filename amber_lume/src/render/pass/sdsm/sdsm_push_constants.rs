use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct SdsmPushConstants {
    pub sdsm_result_buffer_device_address: DeviceAddress,

    pub depth_descriptor_id: u32,
    pub depth_width: u32,
    pub depth_height: u32,

    _pad0: [u32; 27],
}

impl SdsmPushConstants {
    pub fn create(
        sdsm_result_buffer_device_address: DeviceAddress,
        depth_descriptor_id: u32,
        depth_width: u32,
        depth_height: u32,
    ) -> Self {
        Self {
            sdsm_result_buffer_device_address,

            depth_descriptor_id,
            depth_width,
            depth_height,

            _pad0: [0; 27],
        }
    }
}
