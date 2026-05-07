use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::ids::SliceIndex;
use crate::render::buffer::typed::physics_debug_vertex_buffer::PhysicsDebugVertexGPU;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;
use crate::utils::matrix_wrappers::ViewProjectionMatrix;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct PhysicsDebugPushConstants {
    pub view_projection: [[f32; 4]; 4],

    pub physics_debug_vertex_buffer_device_address: DeviceAddress,
    
    _pad0: [u32; 14],
}

impl PhysicsDebugPushConstants {
    pub fn create(
        view_projection: &ViewProjectionMatrix,
        physics_debug_vertex_buffer: BufferView<SliceBuffer<PhysicsDebugVertexGPU>>,
    ) -> Self {
        Self {
            view_projection: view_projection.value.to_cols_array_2d(),

            physics_debug_vertex_buffer_device_address: physics_debug_vertex_buffer.slice_at(SliceIndex::ZERO).device_address(),
            
            _pad0: [0; 14],
        }
    }
}
