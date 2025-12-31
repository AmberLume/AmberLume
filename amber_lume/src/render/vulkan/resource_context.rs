use crate::render::vulkan::buffer::transfer_context::TransferContext;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::{anyhow, Result};
use std::sync::{Arc, Mutex};
use tracing::info;
use crate::render::vulkan::buffer::buffer_manager::BufferManager;

pub struct ResourceContext {
    pub buffer_manager: Arc<BufferManager>,

    pub large_transfer_context: Arc<Mutex<TransferContext>>,
}

impl ResourceContext {
    pub fn create(
        device_context: &mut DeviceContext,
    ) -> Result<Self> {
        let buffer_manager = Arc::new(BufferManager::create(device_context));

        let large_transfer_context = TransferContext::create(device_context, "large", 128 * 1024 * 1024)?;

        Ok(Self {
            buffer_manager,

            large_transfer_context: Arc::new(Mutex::new(large_transfer_context)),
        })
    }

    pub fn destroy(self, device_context: &DeviceContext) -> Result<()> {
        let mut large_transfer_context = self.large_transfer_context.lock().unwrap();
        large_transfer_context.destroy(&device_context)?;

        let buffer_manager = Arc::try_unwrap(self.buffer_manager)
            .map_err(|arc| anyhow!("BufferManager still in use: {}", Arc::strong_count(&arc)))?;
        buffer_manager.destroy()?;

        info!("ResourceContext destroyed");

        Ok(())
    }
}
