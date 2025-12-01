use crate::render::vulkan::buffer::transfer_context::TransferContext;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use std::sync::Arc;

pub struct ResourceContext {
    transfer_context: TransferContext,
}

impl ResourceContext {
    pub fn create(device_context: Arc<DeviceContext>) -> Result<Self> {
        let transfer_context = TransferContext::create(
            device_context.device.clone(),
            device_context.queues.transfer(),
        )?;

        Ok(Self { transfer_context })
    }

    pub fn destroy(&mut self) {
        self.transfer_context.destroy()
    }
}
