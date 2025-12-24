use crate::render::vulkan::buffer::transfer_context::TransferContext;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use tracing::info;

pub struct ResourceContext {
    pub transfer_context: Arc<Mutex<TransferContext>>,
}

impl ResourceContext {
    pub fn create(device_context: &mut DeviceContext) -> Result<Self> {
        let transfer_context = TransferContext::create(device_context, 10 * 1024 * 1024)?;

        Ok(Self {
            transfer_context: Arc::new(Mutex::new(transfer_context)),
        })
    }

    pub fn destroy(&mut self, device_context: &DeviceContext) -> Result<()> {
        let mut transfer_context = self.transfer_context.lock().unwrap();

        transfer_context.destroy(&device_context)?;

        info!("ResourceContext destroyed");

        Ok(())
    }
}
