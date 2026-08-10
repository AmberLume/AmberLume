use gpu::BufferBuilder;
use gpu::ManagedBufferFactory;
use gpu::SliceBuffer;
use gpu_data::MaterialGPU;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use gpu_allocator::MemoryLocation;

pub fn create_materials_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<SliceBuffer<MaterialGPU>> {
    BufferBuilder::slice(capacity).build(
        buffer_factory,
        "materials",
        BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
