use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::buffer_wrapper::buffer_wrapper::BufferWrapper;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk;
use gpu_allocator::MemoryLocation;
use tracing::info;
use vk::{BufferUsageFlags, DeviceSize};

pub struct IndexBuffer {
    pub name: String,
    pub size_of: usize,

    pub buffer: Buffer,
}

impl IndexBuffer {
    pub fn create(device_context: &mut DeviceContext, capacity: usize) -> Result<Self> {
        let size_of = size_of::<u32>();
        let size = (size_of * capacity) as DeviceSize;

        let name = String::from("index_buffer");

        let buffer = Buffer::create(
            device_context,
            size,
            BufferUsageFlags::INDEX_BUFFER | BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
            &name,
        )?;

        Ok(Self {
            name,
            size_of,

            buffer,
        })
    }
}

impl BufferWrapper for IndexBuffer {
    fn allocate_space(&self, count: usize) -> Result<u64> {
        let size = (self.size_of * count) as DeviceSize;

        Ok(self.buffer.allocate_space(size)?)
    }

    fn destroy(&mut self) -> Result<()> {
        self.buffer.destroy()?;

        info!("Buffer '{}' destroyed", self.name);

        Ok(())
    }
}
