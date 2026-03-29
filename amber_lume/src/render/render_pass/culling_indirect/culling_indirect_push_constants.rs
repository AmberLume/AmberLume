use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::ids::SliceIndex;
use crate::render::buffer::typed::culling_views_buffer::CullingViewGPU;
use crate::render::buffer::typed::entity_buffer::EntityGPU;
use crate::render::buffer::typed::mesh_buffer::MeshGPU;
use crate::render::buffer::typed::submesh_buffer::SubmeshGPU;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::factories::buffer::typed_buffer::typed_buffer::TypedBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;
use crate::render::statistics::raw::raw_gpu_render_statistics::RawGpuRenderStatistics;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct CullingIndirectPushConstants {
    pub culling_views_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub mesh_buffer_device_address: DeviceAddress,
    pub submesh_buffer_device_address: DeviceAddress,
    pub gpu_render_stats_buffer_device_address: DeviceAddress,
    pub culling_views_count: u32,
    pub entity_count: u32,
}

impl CullingIndirectPushConstants {
    pub fn create(
        culling_views_buffer: BufferView<SliceBuffer<CullingViewGPU>>,
        entity_buffer: BufferView<SliceBuffer<EntityGPU>>,
        mesh_buffer: BufferView<SliceBuffer<MeshGPU>>,
        submesh_buffer: BufferView<SliceBuffer<SubmeshGPU>>,
        gpu_render_stats_buffer: BufferView<TypedBuffer<RawGpuRenderStatistics>>,
        culling_views_count: u32,
        entity_count: u32,
    ) -> Self {
        Self {
            culling_views_buffer_device_address: culling_views_buffer.slice_at(SliceIndex::ZERO).device_address(),
            entity_buffer_device_address: entity_buffer.slice_at(SliceIndex::ZERO).device_address(),
            mesh_buffer_device_address: mesh_buffer.slice_at(SliceIndex::ZERO).device_address(),
            submesh_buffer_device_address: submesh_buffer.slice_at(SliceIndex::ZERO).device_address(),
            gpu_render_stats_buffer_device_address: gpu_render_stats_buffer.get().device_address(),

            culling_views_count,
            entity_count,
        }
    }
}
