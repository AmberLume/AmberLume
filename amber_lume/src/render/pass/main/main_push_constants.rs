use crate::ids::SliceIndex;
use crate::render::buffer::typed::draw_data_buffer::DrawDataGPU;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;
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
    pub shadow_pcf_radius: i32,

    _pad0: [u32; 13],
}

impl MainPushConstants {
    pub fn create(
        scene_buffer: PhysicalBuffer,
        draw_data_buffer: BufferView<SliceBuffer<DrawDataGPU>>,
        vertex_buffer_device_address: DeviceAddress,
        entity_buffer: PhysicalBuffer,
        submesh_buffer_device_address: DeviceAddress,
        material_buffer_device_address: DeviceAddress,
        bone_transform_buffer_device_address: DeviceAddress,
        shadow_array_descriptor_id: ResourceId,
        shadow_cascades_buffer: PhysicalBuffer,
        shadow_bias: f32,
        shadow_pcf_radius: i32,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer
                .slice_at(SliceIndex::ZERO)
                .device_address(),
            vertex_buffer_device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            submesh_buffer_device_address,
            material_buffer_device_address,
            bone_transform_buffer_device_address,
            shadow_cascades_buffer_device_address: shadow_cascades_buffer.device_address,

            shadow_array_descriptor_id,
            shadow_bias,
            shadow_pcf_radius,

            _pad0: [0; 13],
        }
    }
}
