use render_graph::PhysicalBuffer;
use render_graph::PhysicalReadback;
use index_allocator::ResourceId;
use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct MainPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub vertex_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub submesh_buffer_device_address: DeviceAddress,
    pub material_buffer_device_address: DeviceAddress,
    pub bone_transform_buffer_device_address: DeviceAddress,
    pub picked_entity_buffer_device_address: DeviceAddress,

    pub shadow_factor_descriptor_id: u32,
    pub shadow_enabled: u32,
    pub shadow_colored: u32,

    pub gtao_descriptor_id: u32,
    pub ao_enabled: u32,

    pub sh_descriptor_id: u32,
    pub brdf_lut_descriptor_id: u32,

    pub pick_x: u32,
    pub pick_y: u32,

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
        picked_entity: &PhysicalReadback,
        shadow_factor_descriptor_id: ResourceId,
        shadow_enabled: u32,
        shadow_colored: u32,
        gtao_descriptor_id: ResourceId,
        ao_enabled: u32,
        sh_descriptor_id: ResourceId,
        brdf_lut_descriptor_id: ResourceId,
        pick_x: u32,
        pick_y: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            vertex_buffer_device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            submesh_buffer_device_address,
            material_buffer_device_address,
            bone_transform_buffer_device_address: bone_transform_buffer.device_address,
            picked_entity_buffer_device_address: picked_entity.device_address,

            shadow_factor_descriptor_id: shadow_factor_descriptor_id.inner,
            shadow_enabled,
            shadow_colored,

            gtao_descriptor_id: gtao_descriptor_id.inner,
            ao_enabled,

            sh_descriptor_id: sh_descriptor_id.inner,
            brdf_lut_descriptor_id: brdf_lut_descriptor_id.inner,

            pick_x,
            pick_y,

            _pad0: 0,
        }
    }
}
