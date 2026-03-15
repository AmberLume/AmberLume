use anyhow::Result;
use ash::vk;
use gpu_allocator::MemoryLocation;
use vk::{BufferUsageFlags, DeviceSize};
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::flat_buffer::flat_buffer::FlatBuffer;
use crate::render::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;

pub fn create_renderer_staging_buffer(
    buffer_factory: &ManagedBufferFactory,
    frame_count: u32,
    size: DeviceSize,
) -> Result<FrameBuffer<FlatBuffer>> {
    BufferBuilder::flat(size)
        .per_frame(frame_count)
        .build(
            buffer_factory, 
            "renderer_staging",
            BufferUsageFlags::TRANSFER_SRC, 
            MemoryLocation::CpuToGpu,
        )
}
