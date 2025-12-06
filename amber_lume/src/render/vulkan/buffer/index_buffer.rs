use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::buffer::transfer_context::TransferContext;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk::DeviceAddress;
use ash::{Device, vk};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::Allocator;
use tracing::info;
use vk::{BufferUsageFlags, CommandBuffer, DeviceSize, IndexType};

pub struct IndexBuffer {
    buffer: Buffer,
    index_count: u32,
    index_type: IndexType,
}

impl IndexBuffer {
    pub fn create(
        device_context: &DeviceContext,
        allocator: &mut Allocator,
        capacity: usize,
    ) -> Result<Self> {
        let size = (size_of::<u32>() * capacity) as DeviceSize;

        let buffer = Buffer::create(
            device_context,
            allocator,
            size,
            BufferUsageFlags::INDEX_BUFFER
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
            "index_buffer",
        )?;

        Ok(Self {
            buffer,
            index_count: 0,
            index_type: IndexType::UINT32,
        })
    }

    pub fn upload(
        &mut self,
        transfer_context: &mut TransferContext,
        indices: &[u32],
    ) -> Result<DeviceSize> {
        let offset = transfer_context.copy_to_buffer(&mut self.buffer, indices)?;
        self.index_count = indices.len() as u32;

        Ok(offset)
    }

    pub fn device_address(&self) -> Option<DeviceAddress> {
        self.buffer.device_address
    }

    pub fn index_count(&self) -> u32 {
        self.index_count
    }

    pub fn bind(&self, device: &Device, cmd: CommandBuffer) {
        unsafe {
            device.cmd_bind_index_buffer(cmd, self.buffer.handle, 0, self.index_type);
        }
    }

    pub fn destroy(&mut self) -> Result<()> {
        self.buffer.destroy()?;

        info!("IndexBuffer destroyed");

        Ok(())
    }
}
