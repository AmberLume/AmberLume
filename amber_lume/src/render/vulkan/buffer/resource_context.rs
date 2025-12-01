use crate::render::vulkan::buffer::buffer_manager::BufferManager;
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
    buffer_manager: BufferManager,
}

impl ResourceContext {
    pub fn create(vk_context: Arc<VkContext>, device_context: Arc<DeviceContext>) -> Result<Self> {
        let mut allocator = Self::create_allocator(vk_context.clone(), device_context.clone())?;

        let transfer_context = TransferContext::create(
            device_context.device.clone(),
            &mut allocator,
            device_context.queues.transfer(),
            10 * 1024 * 1024,
        )?;

        let buffer_manager = BufferManager::create(device_context.device.clone(), &mut allocator)?;

        Ok(Self {
            allocator: ManuallyDrop::new(allocator),

            transfer_context,
            buffer_manager,
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
