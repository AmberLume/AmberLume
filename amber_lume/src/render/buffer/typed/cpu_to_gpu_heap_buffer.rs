use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu_allocator::MemoryLocation;
use crate::render::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::factories::buffer::heap_buffer::heap_buffer::HeapBuffer;

pub fn create_cpu_to_gpu_heap_buffer(
    buffer_factory: &ManagedBufferFactory,
    frame_count: u32,
    size: DeviceSize,
) -> Result<FrameBuffer<HeapBuffer>> {
    BufferBuilder::heap(size)
        .per_frame(frame_count)
        .build(
        buffer_factory,
        "cpu_to_gpu_heap",
        BufferUsageFlags::STORAGE_BUFFER 
            | BufferUsageFlags::TRANSFER_DST 
            | BufferUsageFlags::TRANSFER_SRC 
            | BufferUsageFlags::VERTEX_BUFFER
            | BufferUsageFlags::INDEX_BUFFER,
        MemoryLocation::CpuToGpu,
    )
}
