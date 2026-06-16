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
    pub shadow_cascades_buffer_device_address: DeviceAddress,

    pub shadow_array_descriptor_id: ResourceId,
    pub shadow_bias: f32,
    pub shadow_normal_bias: f32,
    pub shadow_pcf_world_radius: f32,
    pub shadow_pcf_sample_count: u32,
    pub shadow_cascade_blend_range: f32,

    pub gtao_descriptor_id: ResourceId,
    pub gtao_enabled: u32,

    pub frame_index: u32,

    _pad0: [u32; 7],
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
        shadow_array_descriptor_id: ResourceId,
        shadow_cascades_buffer: PhysicalBuffer,
        shadow_bias: f32,
        shadow_normal_bias: f32,
        shadow_pcf_world_radius: f32,
        shadow_pcf_sample_count: u32,
        shadow_cascade_blend_range: f32,
        gtao_descriptor_id: ResourceId,
        gtao_enabled: u32,
        frame_index: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            vertex_buffer_device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            submesh_buffer_device_address,
            material_buffer_device_address,
            bone_transform_buffer_device_address: bone_transform_buffer.device_address,
            shadow_cascades_buffer_device_address: shadow_cascades_buffer.device_address,

            shadow_array_descriptor_id,
            shadow_bias,
            shadow_normal_bias,
            shadow_pcf_world_radius,
            shadow_pcf_sample_count,
            shadow_cascade_blend_range,

            gtao_descriptor_id,
            gtao_enabled,

            frame_index,

            _pad0: [0; 7],
        }
    }
}
