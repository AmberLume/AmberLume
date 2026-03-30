use anyhow::Result;
use ash::vk::BufferUsageFlags;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct PhysicsDebugVertexGPU {
    pub point: [f32; 3],

    _pad0: u32,

    pub color: [f32; 4],
}

impl PhysicsDebugVertexGPU {
    pub fn new(point: [f32; 3], color: [f32; 4]) -> Self {
        Self {
            point,

            _pad0: 0,

            color,
        }
    }
}

pub fn create_physics_vertex_debug_buffer(
    buffer_factory: &ManagedBufferFactory,
    frames_in_flight: u32,
    capacity: u32,
) -> Result<FrameBuffer<SliceBuffer<PhysicsDebugVertexGPU>>> {
    BufferBuilder::slice(capacity)
        .per_frame(frames_in_flight)
        .build(
            buffer_factory,
            "physics_debug_vertex",
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            MemoryLocation::CpuToGpu,
        )
}
