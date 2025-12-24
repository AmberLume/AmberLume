use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk;
use ash::vk::{CommandBufferResetFlags, CommandBufferUsageFlags};
use tracing::info;
use vk::{
    CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel,
    CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo,
};

pub struct CommandRecording {
    pub command_pool: CommandPool,
    pub command_buffer: CommandBuffer,
}

impl CommandRecording {
    pub fn create(device_context: &DeviceContext) -> Result<Self> {
        let command_pool = Self::create_command_pool(&device_context)?;

        let command_buffer = Self::allocate_command_buffer(&device_context, &command_pool)?;

        info!("CommandRecording created");

        Ok(Self {
            command_pool,
            command_buffer,
        })
    }

    fn create_command_pool(device_context: &DeviceContext) -> Result<CommandPool> {
        let command_pool_create_info = CommandPoolCreateInfo::default()
            .queue_family_index(device_context.queue_families.graphics)
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = unsafe {
            device_context
                .device
                .create_command_pool(&command_pool_create_info, None)?
        };

        info!("CommandPool created");

        Ok(command_pool)
    }

    fn allocate_command_buffer(
        device_context: &DeviceContext,
        command_pool: &CommandPool,
    ) -> Result<CommandBuffer> {
        let command_buffer_allocate_info = CommandBufferAllocateInfo::default()
            .command_pool(*command_pool)
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = unsafe {
            device_context
                .device
                .allocate_command_buffers(&command_buffer_allocate_info)?[0]
        };

        info!("CommandBuffers allocated");

        Ok(command_buffer)
    }

    pub fn reset_begin_one_time(&self, device_context: &DeviceContext) -> Result<()> {
        unsafe {
            device_context
                .device
                .reset_command_buffer(self.command_buffer, CommandBufferResetFlags::empty())?
        };

        let command_buffer_begin_info =
            CommandBufferBeginInfo::default().flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            device_context
                .device
                .begin_command_buffer(self.command_buffer, &command_buffer_begin_info)?
        };

        Ok(())
    }

    pub fn end_command_recording(&self, device_context: &DeviceContext) -> Result<()> {
        unsafe {
            device_context
                .device
                .end_command_buffer(self.command_buffer)?
        }

        Ok(())
    }

    pub fn destroy(&self, device_context: &DeviceContext) -> Result<()> {
        let device = &device_context.device;

        unsafe { device.free_command_buffers(self.command_pool, &[self.command_buffer]) };
        unsafe { device.destroy_command_pool(self.command_pool, None) };

        info!("CommandRecording destroyed");

        Ok(())
    }
}
