use crate::buffer::transfer_context::TransferContext;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use ash::Device;
use ash::vk::DeviceSize;
use tracing::{error, info};
use crate::buffer::buffer_manager::BufferManager;
use crate::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::queue::queues::Queues;
use index_allocator::ResourceLimits;
use crate::transfer::resource_transfer::ResourceTransfer;
use crate::factories::resource_factories::ResourceFactories;

pub struct ResourceContext {
    pub buffer_manager: Arc<BufferManager>,

    pub resource_transfer: Arc<ResourceTransfer>,

    transfer_context_thread: Option<JoinHandle<()>>,
}

impl ResourceContext {
    pub fn create(
        device: &Device,
        queues: Arc<Queues>,
        resource_factories: Arc<ResourceFactories>,
        resource_limits: &ResourceLimits,
        frames_in_flight: u32,
    ) -> Result<Self> {
        let buffer_manager = BufferManager::create(
            &resource_factories.buffer_factory,
            resource_limits,
            frames_in_flight,
        )?;

        let transfer_context = TransferContext::create(
            device,
            queues,
            "transfer",
            resource_limits.max_staging_size as DeviceSize,
            &resource_factories.buffer_factory,
        )?;
        let transfer_tx = transfer_context.get_sender();

        let resource_transfer = Arc::new(ResourceTransfer::create(
            transfer_tx,
        ));

        let transfer_context_thread = spawn(move || {
            if let Err(e) = transfer_context.flush() {
                error!("Error while flushing transfer context: {:?}", e);
            }

            transfer_context.destroy(&resource_factories.buffer_factory).unwrap();
        });

        Ok(Self {
            buffer_manager: Arc::new(buffer_manager),

            resource_transfer,

            transfer_context_thread: Some(transfer_context_thread),
        })
    }

    pub fn destroy(mut self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        self.resource_transfer.stop()?;

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
