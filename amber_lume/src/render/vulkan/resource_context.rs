use crate::render::vulkan::buffer::transfer_context::TransferContext;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use ash::Device;
use ash::vk::DeviceSize;
use tracing::{error, info};
use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::queue::queues::Queues;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::vulkan::resource_loader::ResourceLoader;
use crate::resources::resource_factories::ResourceFactories;

pub struct ResourceContext {
    pub buffer_manager: Arc<BufferManager>,

    pub resource_loader: Arc<ResourceLoader>,

    transfer_context_thread: Option<JoinHandle<()>>,
}

impl ResourceContext {
    pub fn create(
        device: &Device,
        queues: Arc<Queues>, 
        resource_factories: Arc<ResourceFactories>,
        renderer_limits: &RendererLimits,
    ) -> Result<Self> {
        let buffer_manager = BufferManager::create(&resource_factories.managed_buffer_factory, &renderer_limits)?;

        let transfer_context = TransferContext::create(
            device,
            queues,
            "transfer",
            renderer_limits.buffer_limits.max_staging_size as DeviceSize,
            &resource_factories.managed_buffer_factory,
        )?;
        let transfer_tx = transfer_context.get_sender();

        let resource_loader = Arc::new(ResourceLoader::create(
            transfer_tx,
        ));

        let transfer_context_thread = spawn(move || {
            if let Err(e) = transfer_context.flush() {
                error!("Error while flushing transfer context: {:?}", e);
            }

            transfer_context.destroy(&resource_factories.managed_buffer_factory).unwrap();
        });

        Ok(Self {
            buffer_manager: Arc::new(buffer_manager),

            resource_loader,

            transfer_context_thread: Some(transfer_context_thread),
        })
    }

    pub fn destroy(mut self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        self.resource_loader.stop()?;

        if let Some(handle) = self.transfer_context_thread.take() {
            handle.join().unwrap();
        }

        let buffer_manager = Arc::try_unwrap(self.buffer_manager).map_err(|arc|
            anyhow!("BufferManager still in use: {}", Arc::strong_count(&arc))
        )?;
        buffer_manager.destroy(&buffer_factory)?;

        info!("ResourceContext destroyed");

        Ok(())
    }
}
