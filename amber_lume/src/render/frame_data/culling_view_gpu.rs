use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use glam::Vec4Swizzles;
use crate::ids::SliceIndex;
use crate::render::buffer::typed::draw_data_buffer::DrawDataGPU;
use crate::render::buffer::typed::indirect_buffer::IndirectGPU;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::factories::buffer::typed_buffer::typed_buffer::TypedBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;
use crate::utils::matrix_wrappers::ViewProjectionMatrix;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct CullingViewGPU {
    pub frustum_planes: [[f32; 4]; 6],

    pub indirect_buffer_device_address: DeviceAddress,
    pub draw_count_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,

    _pad0: [u32; 2],
}

impl CullingViewGPU {
    pub fn create(
        view_projection: &ViewProjectionMatrix,
        indirect_buffer: BufferView<SliceBuffer<IndirectGPU>>,
        draw_count_buffer: BufferView<TypedBuffer<u32>>,
        draw_data_buffer: BufferView<SliceBuffer<DrawDataGPU>>,
    ) -> Self {
        let frustum_planes = Self::frustum_planes_from_matrix(&view_projection);

        Self {
            frustum_planes,

            indirect_buffer_device_address: indirect_buffer.slice_at(SliceIndex::ZERO).device_address(),
            draw_count_buffer_device_address: draw_count_buffer.get().device_address(),
            draw_data_buffer_device_address: draw_data_buffer.slice_at(SliceIndex::ZERO).device_address(),

            _pad0: [0; 2],
        }
    }

    pub fn create_for_cascade(
        indirect_buffer: BufferView<SliceBuffer<IndirectGPU>>,
        draw_count_buffer: BufferView<TypedBuffer<u32>>,
        draw_data_buffer: BufferView<SliceBuffer<DrawDataGPU>>,
    ) -> Self {
        Self {
            frustum_planes: [[0.0f32; 4]; 6],

            indirect_buffer_device_address: indirect_buffer.slice_at(SliceIndex::ZERO).device_address(),
            draw_count_buffer_device_address: draw_count_buffer.get().device_address(),
            draw_data_buffer_device_address: draw_data_buffer.slice_at(SliceIndex::ZERO).device_address(),

            _pad0: [0; 2],
        }
    }

    fn frustum_planes_from_matrix(view_projection: &ViewProjectionMatrix) -> [[f32; 4]; 6] {
        let mut planes = [[0.0f32; 4]; 6];

        let combinations = [
            view_projection.value.row(3) + view_projection.value.row(0),
            view_projection.value.row(3) - view_projection.value.row(0),
            view_projection.value.row(3) + view_projection.value.row(1),
            view_projection.value.row(3) - view_projection.value.row(1),
            view_projection.value.row(2),
            view_projection.value.row(3) - view_projection.value.row(2),
        ];

        for (index, plane) in combinations.iter().enumerate() {
            let length = plane.xyz().length();

            let normalized = plane / length;

            planes[index] = normalized.to_array();
        }

        planes
    }
}
