use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use glam::Mat4;
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::buffer::buffer::Buffer;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct SceneGpuData {
    pub projection_matrix: [[f32; 4]; 4],

    pub index_buffer_device_address: u64,
    pub vertex_buffer_device_address: u64,

    pub entity_buffer_device_address: u64,

    pub model_buffer_device_address: u64,
    pub model_availability_buffer_device_address: u64,

    pub primitive_buffer_device_address: u64,
    _pad0: [f32; 2],
}

impl SceneGpuData {
    pub fn create(
        projection_matrix: Mat4,
        index_buffer_device_address: u64,
        vertex_buffer_device_address: u64,
        entity_buffer_device_address: u64,
        model_buffer_device_address: u64,
        model_availability_buffer_device_address: u64,
        primitive_buffer_device_address: u64,
    ) -> Self {
        Self {
            projection_matrix: projection_matrix.to_cols_array_2d(),

            index_buffer_device_address,
            vertex_buffer_device_address,

            entity_buffer_device_address,

            model_buffer_device_address,
            model_availability_buffer_device_address,

            primitive_buffer_device_address,
            _pad0: [0.0; 2],
        }
    }
}

pub fn create_scene_buffer(
    device_context: &mut DeviceContext,
) -> Result<Buffer> {
    let size_of = size_of::<SceneGpuData>() as DeviceSize;

    Buffer::create(
        device_context,
        "scene_buffer",
        size_of,
        0,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::CpuToGpu,
    )
}
