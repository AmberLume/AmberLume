use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceAddress};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec4Swizzles};
use gpu_allocator::MemoryLocation;
use crate::ids::SliceIndex;
use crate::render::buffer::typed::draw_data_buffer::DrawDataGpuData;
use crate::render::buffer::typed::indirect_buffer::IndirectGpuData;
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::factories::buffer::typed_buffer::typed_buffer::TypedBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct CullingViewGpuData {
    pub frustum_planes: [[f32; 4]; 6],

    pub indirect_buffer_device_address: DeviceAddress,
    pub draw_count_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,

    _pad0: [u32; 2],
}

impl CullingViewGpuData {
    pub fn create(
        transform_matrix: Mat4,
        indirect_buffer: BufferView<SliceBuffer<IndirectGpuData>>,
        draw_count_buffer: BufferView<TypedBuffer<u32>>,
        draw_data_buffer: BufferView<SliceBuffer<DrawDataGpuData>>,
    ) -> Self {
        let frustum_planes = Self::frustum_planes_from_matrix(transform_matrix);

        Self {
            frustum_planes,

            indirect_buffer_device_address: indirect_buffer.slice_at(SliceIndex::ZERO).device_address(),
            draw_count_buffer_device_address: draw_count_buffer.get().device_address(),
            draw_data_buffer_device_address: draw_data_buffer.slice_at(SliceIndex::ZERO).device_address(),

            _pad0: [0; 2],
        }
    }

    fn frustum_planes_from_matrix(matrix: Mat4) -> [[f32; 4]; 6] {
        let mut planes = [[0.0f32; 4]; 6];

        let combinations = [
            matrix.row(3) + matrix.row(0),
            matrix.row(3) - matrix.row(0),
            matrix.row(3) + matrix.row(1),
            matrix.row(3) - matrix.row(1),
            matrix.row(2),
            matrix.row(3) - matrix.row(2),
        ];

        for (index, plane) in combinations.iter().enumerate() {
            let length = plane.xyz().length();

            let normalized = plane / length;

            planes[index] = normalized.to_array();
        }

        planes
    }
}

pub fn create_culling_views_buffer(
    buffer_factory: &ManagedBufferFactory,
    frame_count: u32,
    render_view_count: u32,
) -> Result<FrameBuffer<SliceBuffer<CullingViewGpuData>>> {
    BufferBuilder::slice(render_view_count)
        .per_frame(frame_count)
        .build(
            buffer_factory,
            "culling_views_buffer",
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::CpuToGpu,
        )
}
