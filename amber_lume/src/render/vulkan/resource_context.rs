use crate::render::vulkan::buffer::transfer_context::TransferContext;
use anyhow::{anyhow, Result};
use std::sync::{Arc, Mutex};
use ash::Device;
use tracing::{info, warn};
use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::queue::queues::Queues;
use crate::resources::resource_factories::ResourceFactories;

pub struct ResourceContext {
    pub buffer_manager: Arc<BufferManager>,

    pub large_transfer_context: Arc<Mutex<Option<TransferContext>>>,
}

impl ResourceContext {
    pub fn create(
        device: &Device,
        queues: Arc<Queues>, 
        resource_factories: &ResourceFactories,
    ) -> Result<Self> {
        let buffer_manager = BufferManager::create(&resource_factories.managed_buffer_factory);

        let large_transfer_context = TransferContext::create(
            device,
            queues,
            "large", 
            128 * 1024 * 1024,
            &resource_factories.managed_buffer_factory,
        )?;

        Ok(Self {
            buffer_manager: Arc::new(buffer_manager),

            large_transfer_context: Arc::new(Mutex::new(Some(large_transfer_context))),
        })
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        if let Ok(transfer_context_guard) = &mut self.large_transfer_context.lock() {
            if let Some(transfer_context) = transfer_context_guard.take() {
                transfer_context.destroy(&buffer_factory)?;
            } else {
                warn!("Failed to destroy large_transfer_context. It is already None");
            }
        } else {
            warn!("Failed to lock large transfer context to destroy");
        }

        let buffer_manager = Arc::try_unwrap(self.buffer_manager).map_err(|arc|
            anyhow!("BufferManager still in use: {}", Arc::strong_count(&arc))
        )?;
        buffer_manager.destroy(&buffer_factory)?;

        info!("ResourceContext destroyed");

        Ok(())
    }
}
