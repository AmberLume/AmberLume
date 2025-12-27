use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::buffer_wrapper::buffer_wrapper::BufferWrapper;

use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceAddress, DeviceSize};
use gpu_allocator::MemoryLocation;
use tracing::info;
use crate::render::vulkan::data::entity_gpu_data::EntityGpuData;

pub struct EntityBuffer {
    pub name: String,
    pub size_of: usize,

    pub buffer_device_address: DeviceAddress,

    pub buffer: Buffer,
}

impl EntityBuffer {
    pub fn create(device_context: &mut DeviceContext, capacity: usize) -> Result<Self> {
        let size_of = size_of::<EntityGpuData>();
        let name = String::from("entity_buffer");

        let size = (size_of * capacity) as DeviceSize;

        let buffer = Buffer::create(
            device_context,
            size,
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            MemoryLocation::CpuToGpu,
            &name,
        )?;

        Ok(Self {
            name,
            size_of,

            buffer_device_address: buffer.device_address.unwrap(),

            buffer,
        })
    }
}

impl BufferWrapper for EntityBuffer {
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
