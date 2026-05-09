use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::ids::SliceIndex;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;
use crate::render::pass::culling_indirect::render_view_culling_indirect_statistics::CullingIndirectRenderViewStatisticsGPU;
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct CullingIndirectPushConstants {
    pub culling_views_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub mesh_buffer_device_address: DeviceAddress,
    pub submesh_buffer_device_address: DeviceAddress,
    pub meta_statistics_buffer_device_address: DeviceAddress,

    pub view_offset: u32,
    pub view_count: u32,
    pub entity_count: u32,

    _pad0: [u32; 19],
}

impl CullingIndirectPushConstants {
    pub fn create(
        culling_views_buffer: PhysicalBuffer,
        entity_buffer: PhysicalBuffer,
        mesh_buffer_device_address: DeviceAddress,
        submesh_buffer_device_address: DeviceAddress,
        meta_statistics_buffer: BufferView<SliceBuffer<CullingIndirectRenderViewStatisticsGPU>>,
        view_offset: u32,
        view_count: u32,
        entity_count: u32,
    ) -> Self {
        Self {
            culling_views_buffer_device_address: culling_views_buffer.device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            mesh_buffer_device_address,
            submesh_buffer_device_address,
            meta_statistics_buffer_device_address: meta_statistics_buffer.slice_at(SliceIndex::ZERO).device_address(),

            view_offset,
            view_count,
            entity_count,

            _pad0: [0; 19],
        }
    }
}
