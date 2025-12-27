use crate::render::vulkan::buffer::transfer_context::TransferContext;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use tracing::info;

pub struct ResourceContext {
    pub large_transfer_context: Arc<Mutex<TransferContext>>,

    pub small_transfer_context: Arc<Mutex<TransferContext>>,
}

impl ResourceContext {
    pub fn create(device_context: &mut DeviceContext) -> Result<Self> {
        let large_transfer_context =
            TransferContext::create(device_context, "large", 128 * 1024 * 1024)?;

        let small_transfer_context =
            TransferContext::create(device_context, "small", 16 * 1024 * 1024)?;

        Ok(Self {
            large_transfer_context: Arc::new(Mutex::new(large_transfer_context)),

            small_transfer_context: Arc::new(Mutex::new(small_transfer_context)),
        })
    }

    pub fn destroy(&mut self, device_context: &DeviceContext) -> Result<()> {
        let mut small_transfer_context = self.small_transfer_context.lock().unwrap();
        small_transfer_context.destroy(&device_context)?;

        let mut large_transfer_context = self.large_transfer_context.lock().unwrap();
        large_transfer_context.destroy(&device_context)?;

        info!("ResourceContext destroyed");

        Ok(())
    }
}
