use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct TranslucentShadowsPushConstants {
    pub draw_data_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub vertex_buffer_device_address: DeviceAddress,
    pub bone_transform_buffer_device_address: DeviceAddress,
    pub shadow_cascades_buffer_device_address: DeviceAddress,
    pub submesh_buffer_device_address: DeviceAddress,
    pub material_buffer_device_address: DeviceAddress,

    _pad0: [u32; 18],
}

impl TranslucentShadowsPushConstants {
    pub fn create(
        draw_data_buffer: &PhysicalBuffer,
        entity_buffer: &PhysicalBuffer,
        vertex_buffer_device_address: DeviceAddress,
        bone_transform_buffer: &PhysicalBuffer,
        shadow_cascades_buffer: &PhysicalBuffer,
        submesh_buffer_device_address: DeviceAddress,
        material_buffer_device_address: DeviceAddress,
    ) -> Self {
        Self {
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            vertex_buffer_device_address,
            bone_transform_buffer_device_address: bone_transform_buffer.device_address,
            shadow_cascades_buffer_device_address: shadow_cascades_buffer.device_address,
            submesh_buffer_device_address,
            material_buffer_device_address,

            _pad0: [0; 18],
        }
    }
}
