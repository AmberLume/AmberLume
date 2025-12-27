use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::buffer_wrapper::buffer_wrapper::BufferWrapper;
use crate::render::vulkan::buffer_wrapper::staging_buffer::StagingBuffer;
use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::queue::queues::QueueInfo;
use anyhow::{Result, bail};
use ash::vk::{
    CommandPoolCreateFlags, CommandPoolCreateInfo, FenceCreateFlags, FenceCreateInfo,
    SemaphoreCreateInfo, SubmitInfo,
};
use ash::{Device, vk};
use vk::{
    BufferCopy, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandBufferResetFlags, CommandBufferUsageFlags, CommandPool, DeviceSize,
    Fence, Semaphore,
};

pub struct TransferContext {
    device: Device,
    queue_info: QueueInfo,

    command_pool: CommandPool,
    command_buffer: CommandBuffer,

    completion_semaphore: Semaphore,

    completion_fence: Fence,

    staging_buffer: StagingBuffer,
    staging_bytes_offset: DeviceSize,

    in_progress: bool,
}

impl TransferContext {
    pub fn create(
        device_context: &mut DeviceContext,
        tag: &str,
        staging_size: DeviceSize,
    ) -> Result<Self> {
        let staging_buffer = StagingBuffer::create(device_context, &tag, staging_size).unwrap();

        let device = &device_context.device;

        let transfer_queue_info = device_context.queues.transfer();
        let command_pool = Self::create_command_pool(&device_context, &transfer_queue_info)?;

        let semaphore_info = SemaphoreCreateInfo::default();
        let completion_semaphore = unsafe { device.create_semaphore(&semaphore_info, None)? };

        let completion_fence_create_info =
            FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);
        let completion_fence = unsafe { device.create_fence(&completion_fence_create_info, None)? };

        let command_buffer_allocate_info = CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer =
            unsafe { device.allocate_command_buffers(&command_buffer_allocate_info)?[0] };

        Ok(Self {
            device: device.clone(),
            queue_info: transfer_queue_info.clone(),

            command_pool,
            command_buffer,

            completion_semaphore,

            completion_fence,

            staging_buffer,
            staging_bytes_offset: 0,

            in_progress: false,
        })
    }

    fn create_command_pool(
        device_context: &DeviceContext,
        transfer_queue_info: &QueueInfo,
    ) -> Result<CommandPool> {
        let command_pool_create_info = CommandPoolCreateInfo::default()
            .queue_family_index(transfer_queue_info.family)
            .flags(
                CommandPoolCreateFlags::TRANSIENT | CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            );

        let command_pool = unsafe {
            device_context
                .device
                .create_command_pool(&command_pool_create_info, None)?
        };

        Ok(command_pool)
    }

    pub fn begin(&mut self) -> Result<()> {
        if self.in_progress {
            bail!("TransferContext already in progress.");
        }

        let device = &self.device;

        unsafe { device.wait_for_fences(&[self.completion_fence], true, u64::MAX)? };

        unsafe { device.reset_fences(&[self.completion_fence])? };

        unsafe {
            device.reset_command_buffer(self.command_buffer, CommandBufferResetFlags::empty())?
        };

        let begin_info =
            CommandBufferBeginInfo::default().flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe { device.begin_command_buffer(self.command_buffer, &begin_info)? };

        self.staging_bytes_offset = 0;
        self.in_progress = true;

        Ok(())
    }

    pub fn copy_staged_at<T: Copy>(
        &mut self,
        target_buffer: &Buffer,
        target_bytes_offset: DeviceSize,
        data: &[T],
    ) -> Result<DeviceSize> {
        if !self.in_progress {
            bail!("TransferContext is not in progress.");
        }

        let src_bytes_offset = self.staging_bytes_offset;
        let size_bytes = size_of_val(data) as DeviceSize;

        if target_bytes_offset + size_bytes > target_buffer.size {
            bail!("Data exceeds target buffer size.");
        }

        if self.staging_bytes_offset + size_bytes > self.staging_buffer.buffer.size {
            bail!("Data exceeds staging buffer size.");
        }

        self.staging_buffer
            .buffer
            .stage(self.staging_bytes_offset, data)?;
        self.staging_bytes_offset += size_bytes;

        let region = BufferCopy::default()
            .src_offset(src_bytes_offset)
            .dst_offset(target_bytes_offset)
            .size(size_bytes);

        unsafe {
            self.device.cmd_copy_buffer(
                self.command_buffer,
                self.staging_buffer.buffer.handle,
                target_buffer.handle,
                &[region],
            );
        }

        Ok(size_bytes)
    }

    pub fn submit(&mut self) -> Result<()> {
        if !self.in_progress {
            bail!("TransferContext is not in progress.");
        }

        let device = &self.device;

        unsafe { device.end_command_buffer(self.command_buffer)? };

        let buffers = [self.command_buffer];
        let semaphores = [self.completion_semaphore];
        let submit_info = SubmitInfo::default()
            .command_buffers(&buffers)
            .signal_semaphores(&semaphores);

        unsafe {
            device.queue_submit(self.queue_info.queue, &[submit_info], self.completion_fence)?
        };

        unsafe { device.wait_for_fences(&[self.completion_fence], true, u64::MAX)? };

        self.in_progress = false;

        Ok(())
    }

    pub fn destroy(&mut self, device_context: &DeviceContext) -> Result<()> {
        let device = &device_context.device;

        self.staging_buffer.destroy()?;

        unsafe { device.queue_wait_idle(self.queue_info.queue)? };

        unsafe { device.destroy_fence(self.completion_fence, None) };

        unsafe { device.destroy_semaphore(self.completion_semaphore, None) };

        unsafe { device.free_command_buffers(self.command_pool, &[self.command_buffer]) };
        unsafe { device.destroy_command_pool(self.command_pool, None) };

        Ok(())
    }
}
