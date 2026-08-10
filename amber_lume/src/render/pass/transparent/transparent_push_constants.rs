use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use render_graph::PhysicalBuffer;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct TransparentPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub vertex_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub submesh_buffer_device_address: DeviceAddress,
    pub material_buffer_device_address: DeviceAddress,
    pub bone_transform_buffer_device_address: DeviceAddress,

    pub sh_descriptor_id: u32,
    pub brdf_lut_descriptor_id: u32,
}

impl TransparentPushConstants {
    pub fn create(
        scene_buffer: &PhysicalBuffer,
        draw_data_buffer: &PhysicalBuffer,
        vertex_buffer_device_address: DeviceAddress,
        entity_buffer: &PhysicalBuffer,
        submesh_buffer_device_address: DeviceAddress,
        material_buffer_device_address: DeviceAddress,
        bone_transform_buffer: &PhysicalBuffer,
        sh_descriptor_id: u32,
        brdf_lut_descriptor_id: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            vertex_buffer_device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            submesh_buffer_device_address,
            material_buffer_device_address,
            bone_transform_buffer_device_address: bone_transform_buffer.device_address,

            sh_descriptor_id,
            brdf_lut_descriptor_id,
        }
    }
}
