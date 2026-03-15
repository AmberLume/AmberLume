use anyhow::Result;
use ash::vk::BufferUsageFlags;
use gpu_allocator::MemoryLocation;
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;

pub fn create_ui_index_buffer(
    buffer_factory: &ManagedBufferFactory,
    frame_count: u32,
    capacity: u32,
) -> Result<FrameBuffer<SliceBuffer<u32>>> {
    BufferBuilder::slice(capacity)
        .per_frame(frame_count)
        .build(
            buffer_factory,
            "ui_index_buffer",
            BufferUsageFlags::INDEX_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::CpuToGpu,
        )
}
