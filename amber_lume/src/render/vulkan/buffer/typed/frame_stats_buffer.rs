use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::vulkan::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::vulkan::factories::buffer::typed_buffer::typed_buffer::TypedBuffer;
use crate::render::vulkan::renderer::stats::gpu_render_stats::GpuRenderStats;

pub fn create_render_stats_buffer(
    buffer_factory: &ManagedBufferFactory,
    frame_count: u32,
) -> Result<FrameBuffer<TypedBuffer<GpuRenderStats>>> {
    BufferBuilder::typed()
        .per_frame(frame_count)
        .build(
            buffer_factory,
            "render_stats",
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuToCpu,
        )
}
