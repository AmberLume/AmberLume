use crate::buffer::transfer_context::TransferContext;
use anyhow::Result;
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use ash::Device;
use ash::vk::DeviceSize;
use tracing::{error, info};
use crate::queue::queues::Queues;
use index_allocator::ResourceLimits;
use crate::transfer::resource_transfer::ResourceTransfer;
use crate::factories::resource_factories::ResourceFactories;

pub struct ResourceContext {
    pub resource_transfer: Arc<ResourceTransfer>,

    transfer_context_thread: Option<JoinHandle<()>>,
}

impl ResourceContext {
    pub fn create(
        device: &Device,
        queues: Arc<Queues>,
        resource_factories: Arc<ResourceFactories>,
        resource_limits: &ResourceLimits,
    ) -> Result<Self> {
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
            resource_transfer,

            transfer_context_thread: Some(transfer_context_thread),
        })
    }

    pub fn destroy(mut self) -> Result<()> {
        self.resource_transfer.stop()?;

        if let Some(handle) = self.transfer_context_thread.take() {
            handle.join().unwrap();
        }

        info!("ResourceContext destroyed");

        Ok(())
    }
}
