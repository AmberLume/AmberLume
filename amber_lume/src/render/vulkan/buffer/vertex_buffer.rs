use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::buffer::transfer_context::TransferContext;
use crate::render::vulkan::data::vertex::Vertex;
use anyhow::Result;
use ash::{Device, vk};
use gpu_allocator::MemoryLocation::GpuOnly;
use gpu_allocator::vulkan::Allocator;
use vk::{BufferUsageFlags, DeviceAddress, DeviceSize};

pub struct VertexBuffer {
    buffer: Buffer,
    vertex_count: u32,
}

impl VertexBuffer {
    pub fn create(device: Device, allocator: &mut Allocator, capacity: usize) -> Result<Self> {
        let size = (size_of::<Vertex>() * capacity) as DeviceSize;

        let buffer = Buffer::create(
            device,
            allocator,
            size,
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::TRANSFER_DST,
            GpuOnly,
            "vertex_buffer",
        )?;

        Ok(Self {
            buffer,
            vertex_count: 0,
        })
    }

    pub fn upload(
        &mut self,
        transfer_context: &mut TransferContext,
        vertices: &[Vertex],
    ) -> Result<DeviceSize> {
        let offset = transfer_context.copy_to_buffer(&mut self.buffer, vertices)?;
        self.vertex_count = vertices.len() as u32;

        Ok(offset)
    }

    pub fn device_address(&self) -> Option<DeviceAddress> {
        self.buffer.device_address
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }

    pub fn destroy(&mut self) {
        self.buffer.destroy();
    }
}
