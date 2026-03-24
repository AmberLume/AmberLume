use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::ids::SliceIndex;
use crate::render::buffer::typed::draw_data_buffer::DrawDataGpuData;
use crate::render::buffer::typed::entity_buffer::EntityGpuData;
use crate::render::buffer::typed::scene_buffer::SceneGpuData;
use crate::render::buffer::typed::vertex_buffer::VertexGpuData;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::factories::buffer::typed_buffer::typed_buffer::TypedBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShadowsPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub vertex_buffer_device_address: DeviceAddress,

    pub shadow_cascade_index: u32,
    _pad0: u32,
}

impl ShadowsPushConstants {
    pub fn create(
        scene_buffer: BufferView<TypedBuffer<SceneGpuData>>,
        draw_data_buffer: BufferView<SliceBuffer<DrawDataGpuData>>,
        entity_buffer: BufferView<SliceBuffer<EntityGpuData>>,
        vertex_buffer: BufferView<SliceBuffer<VertexGpuData>>,
        shadow_cascade_index: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.get().device_address(),
            draw_data_buffer_device_address: draw_data_buffer.slice_at(SliceIndex::ZERO).device_address(),
            entity_buffer_device_address: entity_buffer.slice_at(SliceIndex::ZERO).device_address(),
            vertex_buffer_device_address: vertex_buffer.slice_at(SliceIndex::ZERO).device_address(),
            
            shadow_cascade_index,
            _pad0: 0,
        }
    }
}
