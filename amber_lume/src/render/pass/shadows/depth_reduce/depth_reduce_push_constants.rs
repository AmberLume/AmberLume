use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use render_graph::PhysicalBuffer;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct DepthReducePushConstants {
    pub result_buffer_device_address: DeviceAddress,

    pub depth_descriptor_id: u32,
    pub depth_width: u32,
    pub depth_height: u32,
    pub stride: u32,

    _pad0: [u32; 26],
}

impl DepthReducePushConstants {
    pub fn create(
        result_buffer: PhysicalBuffer,
        depth_descriptor_id: u32,
        depth_width: u32,
        depth_height: u32,
        stride: u32,
    ) -> Self {
        Self {
            result_buffer_device_address: result_buffer.device_address,

            depth_descriptor_id,
            depth_width,
            depth_height,
            stride,

            _pad0: [0; 26],
        }
    }
}
