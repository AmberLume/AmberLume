use crate::render::vulkan::buffer::transfer_context::TransferContext;
use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::vk_context::VkContext;
use anyhow::Result;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use std::mem::ManuallyDrop;
use std::sync::Arc;
use tracing::info;

pub struct ResourceContext {
    pub allocator: ManuallyDrop<Allocator>,

    transfer_context: TransferContext,
}

impl ResourceContext {
    pub fn create(vk_context: Arc<VkContext>, device_context: Arc<DeviceContext>) -> Result<Self> {
        let allocator = Self::create_allocator(vk_context.clone(), device_context.clone())?;

        let transfer_context = TransferContext::create(
            device_context.device.clone(),
            device_context.queues.transfer(),
        )?;

        Ok(Self {
            allocator: ManuallyDrop::new(allocator),

            transfer_context,
        })
    }

    fn create_allocator(
        vk_context: Arc<VkContext>,
        device_context: Arc<DeviceContext>,
    ) -> Result<Allocator> {
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: vk_context.instance.clone(),
            device: device_context.device.clone(),
            physical_device: device_context.physical_device_info.handle.clone(),
            debug_settings: Default::default(),
            buffer_device_address: true,
            allocation_sizes: Default::default(),
        })?;
        info!("Memory allocator created");

        Ok(allocator)
    }

    pub fn destroy(&mut self) {
        self.transfer_context.destroy();
        unsafe {
            ManuallyDrop::drop(&mut self.allocator);
        }
    }
}
