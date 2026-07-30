use gpu::BufferBuilder;
use gpu::FrameBuffer;
use gpu::ManagedBufferFactory;
use gpu::SliceBuffer;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use yakui::paint::Vertex;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct UiVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

pub fn create_ui_vertex_buffer(
    buffer_factory: &ManagedBufferFactory,
    frame_count: u32,
    capacity: u32,
) -> Result<FrameBuffer<SliceBuffer<Vertex>>> {
    BufferBuilder::slice(capacity).per_frame(frame_count).build(
        buffer_factory,
        "ui_vertex",
        BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::CpuToGpu,
    )
}
