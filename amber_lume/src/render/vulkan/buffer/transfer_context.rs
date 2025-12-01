use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::queue::queues::QueueInfo;
use anyhow::{Result, bail};
use ash::vk::{CommandPoolCreateFlags, CommandPoolCreateInfo, SemaphoreCreateInfo, SubmitInfo};
use ash::{Device, vk};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::Allocator;
use vk::{
    BufferCopy, BufferUsageFlags, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandBufferResetFlags, CommandBufferUsageFlags, CommandPool, DeviceSize,
    Fence, Semaphore,
};

pub struct TransferContext {
    pub device: Device,
    queue_info: QueueInfo,

    command_pool: CommandPool,
    command_buffer: CommandBuffer,

    pub completion_semaphore: Semaphore,

    staging_buffer: Buffer,

    in_progress: bool,
}

impl TransferContext {
    pub fn create(
        device: Device,
        allocator: &mut Allocator,
        queue_info: &QueueInfo,
        staging_size: DeviceSize,
    ) -> Result<Self> {
        let staging = Buffer::create(
            device.clone(),
            allocator,
            staging_size,
            BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
            "staging",
        )?;

        let command_pool = Self::create_command_pool(&device, &queue_info)?;

        let semaphore_info = SemaphoreCreateInfo::default();
        let completion_semaphore = unsafe { device.create_semaphore(&semaphore_info, None)? };

        let command_buffer_allocate_info = CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer =
            unsafe { device.allocate_command_buffers(&command_buffer_allocate_info)?[0] };

        Ok(Self {
            device,
            command_pool,
            queue_info: queue_info.clone(),
            staging_buffer: staging,
            command_buffer,

            completion_semaphore,

            in_progress: false,
        })
    }

    fn create_command_pool(device: &Device, queue_info: &QueueInfo) -> Result<CommandPool> {
        let command_pool_create_info = CommandPoolCreateInfo::default()
            .queue_family_index(queue_info.family)
            .flags(CommandPoolCreateFlags::TRANSIENT);

        let command_pool = unsafe { device.create_command_pool(&command_pool_create_info, None)? };

        Ok(command_pool)
    }

    pub fn begin(&mut self) -> Result<()> {
        if self.in_progress {
            bail!("TransferContext already in progress.");
        }

        let begin_info =
            CommandBufferBeginInfo::default().flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .begin_command_buffer(self.command_buffer, &begin_info)?;
        }

        self.in_progress = true;
        Ok(())
    }

    pub fn copy_to_buffer<T: Copy>(
        &mut self,
        target_buffer: &mut Buffer,
        data: &[T],
    ) -> Result<DeviceSize> {
        if !self.in_progress {
            bail!("TransferContext is not in progress.");
        }

        let size = size_of_val(data) as DeviceSize;

        if target_buffer.offset + size > target_buffer.size {
            bail!("Data exceeds target buffer size.");
        }

        self.staging_buffer.copy_from_slice(data)?;

        let region = BufferCopy::default()
            .src_offset(0)
            .dst_offset(target_buffer.offset)
            .size(size);

        target_buffer.offset = target_buffer.offset + size;

        unsafe {
            self.device.cmd_copy_buffer(
                self.command_buffer,
                self.staging_buffer.handle,
                target_buffer.handle,
                &[region],
            );
        }

        Ok(target_buffer.offset)
    }

    pub fn submit(&mut self) -> Result<()> {
        if !self.in_progress {
            bail!("TransferContext is not in progress.");
        }

        unsafe {
            self.device.end_command_buffer(self.command_buffer)?;

            let buffers = [self.command_buffer];
            let semaphores = [self.completion_semaphore];
            let submit_info = SubmitInfo::default()
                .command_buffers(&buffers)
                .signal_semaphores(&semaphores);

            self.device
                .queue_submit(self.queue_info.queue, &[submit_info], Fence::null())?;

            self.device
                .reset_command_buffer(self.command_buffer, CommandBufferResetFlags::empty())?;
        }

        self.in_progress = false;
        Ok(())
    }

    pub fn destroy(&mut self) {
        unsafe {
            self.device.queue_wait_idle(self.queue_info.queue).unwrap();
        }

        unsafe {
            self.device
                .destroy_semaphore(self.completion_semaphore, None);
        }
        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
        }
    }
}
