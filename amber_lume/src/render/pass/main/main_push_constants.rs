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

    pub shadow_mask_resource_id: ResourceId,

    _pad0: [u32; 17],
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
        shadow_mask_resource_id: ResourceId,
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

            shadow_mask_resource_id,

            _pad0: [0; 17],
        }
    }
}
