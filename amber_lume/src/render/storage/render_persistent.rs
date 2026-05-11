use anyhow::Result;
use ash::vk::DeviceSize;
use crate::limits::AmberLumeLimits;
use crate::render::buffer::typed::cpu_to_gpu_heap_buffer::create_cpu_to_gpu_heap_buffer;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;

pub struct RenderPersistent {
    pub cpu_to_gpu_allocator: HeapAllocator,
}

impl RenderPersistent {
    pub fn new(
        buffer_factory: &ManagedBufferFactory,
        limits: &AmberLumeLimits,
    ) -> Result<Self> {
        let cpu_to_gpu_buffer = create_cpu_to_gpu_heap_buffer(
            buffer_factory,
            limits.frames_in_flight,
            limits.resource_limits.max_frame_heap_size as DeviceSize,
        )?;
        let cpu_to_gpu_allocator = HeapAllocator::create(
            cpu_to_gpu_buffer.into_managed_buffer(),
            limits.resource_limits.max_frame_heap_size as DeviceSize,
            limits.frames_in_flight,
        )?;

        Ok(Self {
            cpu_to_gpu_allocator,
        })
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        self.cpu_to_gpu_allocator.destroy(buffer_factory)
    }
}
