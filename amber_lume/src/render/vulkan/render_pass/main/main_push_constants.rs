use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::resources::dynamic::resource_provider::ResourceId;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct MainPushConstants {
    pub projection_matrix: [[f32; 4]; 4],
    pub light_direction: [f32; 3],
    _pad0: u32,

    pub draw_data_buffer_device_address: DeviceAddress,
    pub vertex_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub submesh_buffer_device_address: DeviceAddress,
    pub material_buffer_device_address: DeviceAddress,
    
    pub shadow_mask_resource_id: ResourceId,

    _pad1: u32,
}

impl MainPushConstants {
    pub fn create(
        projection_matrix: [[f32; 4]; 4],
        light_direction: [f32; 3],
        draw_data_buffer_device_address: DeviceAddress,
        vertex_buffer_device_address: DeviceAddress,
        entity_buffer_device_address: DeviceAddress,
        submesh_buffer_device_address: DeviceAddress,
        material_buffer_device_address: DeviceAddress,
        shadow_mask_resource_id: ResourceId,
    ) -> Self {
        Self {
            projection_matrix,
            light_direction,
            _pad0: 0,

            draw_data_buffer_device_address,
            vertex_buffer_device_address,
            entity_buffer_device_address,
            submesh_buffer_device_address,
            material_buffer_device_address,

            shadow_mask_resource_id,

            _pad1: 0,
        }
    }
}
