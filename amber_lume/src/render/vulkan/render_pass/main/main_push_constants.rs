use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct MainPushConstants {
    pub projection_matrix: [[f32; 4]; 4],

    pub draw_data_buffer_device_address: DeviceAddress,
    pub vertex_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub submesh_buffer_device_address: DeviceAddress,
    pub material_buffer_device_address: DeviceAddress,
}

impl MainPushConstants {
    pub fn create(
        projection_matrix: [[f32; 4]; 4],
        draw_data_buffer_device_address: DeviceAddress,
        vertex_buffer_device_address: DeviceAddress,
        entity_buffer_device_address: DeviceAddress,
        submesh_buffer_device_address: DeviceAddress,
        material_buffer_device_address: DeviceAddress,
    ) -> Self {
        Self {
            projection_matrix,

            draw_data_buffer_device_address,
            vertex_buffer_device_address,
            entity_buffer_device_address,
            submesh_buffer_device_address,
            material_buffer_device_address,
        }
    }
}
