use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use gpu::BufferRange;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct EntityMotionGPU {
    pub previous_transform_matrix: [[f32; 4]; 4],
    pub previous_vertex_buffer_device_address: DeviceAddress,
    _pad0: [u32; 2],
}

impl EntityMotionGPU {
    pub fn create(previous_transform_matrix: Mat4, previous_vertex_buffer: BufferRange) -> Self {
        Self {
            previous_transform_matrix: previous_transform_matrix.to_cols_array_2d(),
            previous_vertex_buffer_device_address: previous_vertex_buffer.device_address,
            _pad0: [0; 2],
        }
    }
}
