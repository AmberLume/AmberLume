use gpu::BufferBuilder;
use gpu::FrameBuffer;
use gpu::ManagedBufferFactory;
use gpu::SliceBuffer;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use gpu_allocator::MemoryLocation;

pub fn create_ui_index_buffer(
    buffer_factory: &ManagedBufferFactory,
    frame_count: u32,
    capacity: u32,
) -> Result<FrameBuffer<SliceBuffer<u32>>> {
    BufferBuilder::slice(capacity).per_frame(frame_count).build(
        buffer_factory,
        "ui_index",
        BufferUsageFlags::INDEX_BUFFER | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::CpuToGpu,
    )
}
