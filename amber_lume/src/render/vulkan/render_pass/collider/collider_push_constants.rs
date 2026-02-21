use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ColliderPushConstants {
    pub projection_matrix: [[f32; 4]; 4],
    
    pub collider_buffer_device_address: DeviceAddress,
}

impl ColliderPushConstants {
    pub fn create(
        projection_matrix: [[f32; 4]; 4],
        collider_buffer_device_address: DeviceAddress,
    ) -> Self {
        Self {
            projection_matrix,
            
            collider_buffer_device_address,
        }
    }
}
