use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct DepthPushConstants {
    pub view_projection: [[f32; 4]; 4],
    pub vertex_buffer_address: DeviceAddress,
}

impl DepthPushConstants {
    pub fn create(view_projection: [[f32; 4]; 4], vertex_buffer_address: u64) -> Self {
        Self {
            view_projection,
            vertex_buffer_address,
        }
    }
}
