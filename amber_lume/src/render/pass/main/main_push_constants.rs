use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;
use crate::resources::store::providers::resource_provider::ResourceId;
use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct MainPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub vertex_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub submesh_buffer_device_address: DeviceAddress,
    pub material_buffer_device_address: DeviceAddress,
    pub bone_transform_buffer_device_address: DeviceAddress,

    pub shadow_factor_descriptor_id: ResourceId,
    pub shadow_enabled: u32,
    pub shadow_colored: u32,

    pub gtao_descriptor_id: ResourceId,
    pub ao_enabled: u32,

    pub sh_descriptor_id: ResourceId,
    pub brdf_lut_descriptor_id: ResourceId,

    _pad0: u32,
}

impl MainPushConstants {
    pub fn create(
        scene_buffer: PhysicalBuffer,
        draw_data_buffer: PhysicalBuffer,
        vertex_buffer_device_address: DeviceAddress,
        entity_buffer: PhysicalBuffer,
        submesh_buffer_device_address: DeviceAddress,
        material_buffer_device_address: DeviceAddress,
        bone_transform_buffer: PhysicalBuffer,
        shadow_factor_descriptor_id: ResourceId,
        shadow_enabled: u32,
        shadow_colored: u32,
        gtao_descriptor_id: ResourceId,
        ao_enabled: u32,
        sh_descriptor_id: ResourceId,
        brdf_lut_descriptor_id: ResourceId,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            vertex_buffer_device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            submesh_buffer_device_address,
            material_buffer_device_address,
            bone_transform_buffer_device_address: bone_transform_buffer.device_address,

            shadow_factor_descriptor_id,
            shadow_enabled,
            shadow_colored,

            gtao_descriptor_id,
            ao_enabled,

            sh_descriptor_id,
            brdf_lut_descriptor_id,

            _pad0: 0,
        }
    }
}
