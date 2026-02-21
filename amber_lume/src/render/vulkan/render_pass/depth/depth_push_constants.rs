use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct DepthPushConstants {
    pub projection_matrix: [[f32; 4]; 4],

    pub entity_buffer_device_address: DeviceAddress,
    pub vertex_buffer_device_address: DeviceAddress,
}

impl DepthPushConstants {
    pub fn create(
        projection_matrix: [[f32; 4]; 4],
        entity_buffer_device_address: DeviceAddress,
        vertex_buffer_device_address: DeviceAddress,
    ) -> Self {
        Self {
            projection_matrix,

            entity_buffer_device_address,
            vertex_buffer_device_address,
        }
    }
}
