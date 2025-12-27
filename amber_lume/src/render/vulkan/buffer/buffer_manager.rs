use crate::render::vulkan::buffer_wrapper::buffer_wrapper::BufferWrapper;
use crate::render::vulkan::buffer_wrapper::index_buffer::IndexBuffer;
use crate::render::vulkan::buffer_wrapper::entity_buffer::EntityBuffer;
use crate::render::vulkan::buffer_wrapper::vertex_buffer::VertexBuffer;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk::DeviceAddress;
use std::sync::{Arc, Mutex};
use tracing::info;

pub struct BufferManager {
    pub index_buffer: Arc<Mutex<IndexBuffer>>,

    pub vertex_buffer: Arc<Mutex<VertexBuffer>>,
    pub vertex_buffer_device_address: DeviceAddress,

    pub entity_buffer: Arc<Mutex<EntityBuffer>>,
    pub entity_buffer_device_address: DeviceAddress,
}

impl BufferManager {
    pub fn create(device_context: &mut DeviceContext) -> Self {
        let index_buffer = IndexBuffer::create(device_context, 500_000).unwrap();

        let vertex_buffer = VertexBuffer::create(device_context, 1_500_000).unwrap();
        let vertex_buffer_device_address = vertex_buffer.buffer_device_address;

        let entity_buffer = EntityBuffer::create(device_context, 100_000).unwrap();
        let entity_buffer_device_address = entity_buffer.buffer_device_address;

        Self {
            index_buffer: Arc::new(Mutex::new(index_buffer)),

            vertex_buffer: Arc::new(Mutex::new(vertex_buffer)),
            vertex_buffer_device_address,

            entity_buffer: Arc::new(Mutex::new(entity_buffer)),
            entity_buffer_device_address,
        }
    }

    pub fn destroy(&mut self) -> Result<()> {
        self.entity_buffer.lock().unwrap().destroy()?;
        self.vertex_buffer.lock().unwrap().destroy()?;
        self.index_buffer.lock().unwrap().destroy()?;

        info!("BufferManager destroyed");

        Ok(())
    }
}
