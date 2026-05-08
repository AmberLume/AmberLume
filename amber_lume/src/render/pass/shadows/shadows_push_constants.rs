use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::ids::SliceIndex;
use crate::render::buffer::typed::draw_data_buffer::DrawDataGPU;
use crate::render::buffer::typed::entity_buffer::EntityGPU;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShadowsPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub vertex_buffer_device_address: DeviceAddress,
    pub bone_transform_buffer_device_address: DeviceAddress,

    pub shadow_cascade_index: u32,

    _pad0: [u32; 21],
}

impl ShadowsPushConstants {
    pub fn create(
        scene_buffer: &PhysicalBuffer,
        draw_data_buffer: BufferView<SliceBuffer<DrawDataGPU>>,
        entity_buffer: BufferView<SliceBuffer<EntityGPU>>,
        vertex_buffer_device_address: DeviceAddress,
        bone_transform_buffer_device_address: DeviceAddress,
        shadow_cascade_index: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer.slice_at(SliceIndex::ZERO).device_address(),
            entity_buffer_device_address: entity_buffer.slice_at(SliceIndex::ZERO).device_address(),
            vertex_buffer_device_address,
            bone_transform_buffer_device_address,

            shadow_cascade_index,

            _pad0: [0; 21],
        }
    }
}
