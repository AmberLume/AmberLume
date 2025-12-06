use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::buffer::transfer_context::TransferContext;
use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::vulkan_context::VulkanContext;
use anyhow::Result;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use std::mem::ManuallyDrop;
use tracing::info;

pub struct ResourceContext {
    pub allocator: ManuallyDrop<Allocator>,

    transfer_context: TransferContext,
    buffer_manager: BufferManager,
}

impl ResourceContext {
    pub fn create(vulkan_context: &VulkanContext, device_context: &DeviceContext) -> Result<Self> {
        let mut allocator = Self::create_allocator(&vulkan_context, &device_context)?;

        let transfer_context =
            TransferContext::create(&device_context, &mut allocator, 10 * 1024 * 1024)?;

        let buffer_manager = BufferManager::create(&device_context, &mut allocator)?;

        Ok(Self {
            allocator: ManuallyDrop::new(allocator),

            transfer_context,
            buffer_manager,
        })
    }

    fn create_allocator(
        vulkan_context: &VulkanContext,
        device_context: &DeviceContext,
    ) -> Result<Allocator> {
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: vulkan_context.instance.clone(),
            device: device_context.device.clone(),
            physical_device: device_context.physical_device_info.handle.clone(),
            debug_settings: Default::default(),
            buffer_device_address: true,
            allocation_sizes: Default::default(),
        })?;
        info!("Memory allocator created");

        Ok(allocator)
    }

    pub fn destroy(&mut self, device_context: &DeviceContext) -> Result<()> {
        self.transfer_context.destroy(&device_context)?;

        self.buffer_manager.destroy()?;

        unsafe { ManuallyDrop::drop(&mut self.allocator) };

        info!("ResourceContext destroyed");

        Ok(())
    }
}
