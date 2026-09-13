use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use gpu::BufferRange;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct EntityGPU {
    pub transform_matrix: [[f32; 4]; 4],
    pub vertex_buffer_device_address: DeviceAddress,
    pub vertex_attribute_buffer_device_address: DeviceAddress,
    pub mesh_index: u32,
    _pad0: [u32; 3],
}

impl EntityGPU {
    pub fn create(
        transform_matrix: Mat4,
        mesh_index: u32,
        vertex_buffer: BufferRange,
        vertex_attribute_buffer: BufferRange,
    ) -> Self {
        Self {
            transform_matrix: transform_matrix.to_cols_array_2d(),
            vertex_buffer_device_address: vertex_buffer.device_address,
            vertex_attribute_buffer_device_address: vertex_attribute_buffer.device_address,
            mesh_index,
            _pad0: [0; 3],
        }
    }
}
