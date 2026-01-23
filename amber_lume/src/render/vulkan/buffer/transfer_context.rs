use std::sync::Arc;
use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::{Result, bail};
use ash::vk::{AccessFlags, BufferImageCopy, CommandPoolCreateFlags, CommandPoolCreateInfo, DependencyFlags, FenceCreateFlags, FenceCreateInfo, Image, ImageAspectFlags, ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, PipelineStageFlags, SubmitInfo, QUEUE_FAMILY_IGNORED};
use ash::{Device, vk};
use vk::{
    BufferCopy, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandBufferResetFlags, CommandBufferUsageFlags, CommandPool, DeviceSize,
    Fence,
};
use crate::render::vulkan::buffer::typed::staging_buffer::create_staging_buffer;
use crate::render::vulkan::queue::queues::Queues;

pub struct TransferContext {
    device: Device,
    queues: Arc<Queues>,

    command_pool: CommandPool,
    command_buffer: CommandBuffer,

    completion_fence: Fence,

    staging_buffer: Buffer,
    staging_offset: usize,

    in_progress: bool,
}

impl TransferContext {
    pub fn create(
        device_context: &mut DeviceContext,
        tag: &str,
        staging_size: DeviceSize,
    ) -> Result<Self> {
        let staging_buffer = create_staging_buffer(device_context, &tag, staging_size as usize).unwrap();

        let device = &device_context.device;

        let command_pool = Self::create_command_pool(&device_context)?;

        let completion_fence_create_info = FenceCreateInfo::default()
            .flags(FenceCreateFlags::SIGNALED);
        let completion_fence = unsafe { device.create_fence(&completion_fence_create_info, None)? };

        let command_buffer_allocate_info = CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer =
            unsafe { device.allocate_command_buffers(&command_buffer_allocate_info)?[0] };

        Ok(Self {
            device: device.clone(),
            queues: device_context.queues.clone(),

            command_pool,
            command_buffer,

            completion_fence,

            staging_buffer,
            staging_offset: 0,

            in_progress: false,
        })
    }

    fn create_command_pool(
        device_context: &DeviceContext,
    ) -> Result<CommandPool> {
        let command_pool_create_info = CommandPoolCreateInfo::default()
            .queue_family_index(device_context.queues.transfer_queue_family())
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

        let begin_info = CommandBufferBeginInfo::default()
            .flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe { device.begin_command_buffer(self.command_buffer, &begin_info)? };

        self.staging_offset = 0;
        self.in_progress = true;

        Ok(())
    }

    pub fn copy_staged_at<T: Copy>(
        &mut self,
        target_buffer: &Buffer,
        target_offset: usize,
        data: &[T],
    ) -> Result<usize> {
        if !self.in_progress {
            bail!("TransferContext is not in progress.");
        }

        let src_bytes_offset = self.staging_buffer.size_of_item * self.staging_offset as DeviceSize;
        let size = data.len();

        if target_offset + size > target_buffer.capacity {
            bail!("Data exceeds target buffer size.");
        }

        if self.staging_offset + size > self.staging_buffer.capacity {
            bail!("Data exceeds staging buffer size.");
        }

        let size_bytes = target_buffer.size_of_item * size as DeviceSize;

        self.staging_buffer.stage(self.staging_offset, data)?;
        self.staging_offset += size_bytes as usize;

        let region = BufferCopy::default()
            .src_offset(src_bytes_offset)
            .dst_offset(target_buffer.size_of_item * target_offset as DeviceSize)
            .size(size_bytes);

        unsafe {
            self.device.cmd_copy_buffer(
                self.command_buffer,
                self.staging_buffer.handle,
                target_buffer.handle,
                &[region],
            );
        }

        Ok(size)
    }

    pub fn get_current_offset(&self) -> usize {
        self.staging_offset
    }

    pub fn stage(
        &mut self,
        data: &[u8],
    ) -> Result<()> {
        if !self.in_progress {
            bail!("TransferContext is not in progress.");
        }

        let size_bytes = size_of_val(data);

        if (self.staging_offset + size_bytes) as DeviceSize > self.staging_buffer.get_size_bytes() {
            bail!("Data exceeds staging buffer size.");
        }

        self.staging_buffer.stage(self.staging_offset, data)?;
        self.staging_offset += size_bytes;

        Ok(())
    }

    pub fn copy_stage_to_image(
        &mut self,
        image: Image,
        mip_levels: u32,
        copies: &[BufferImageCopy],
    ) -> Result<()> {
        if !self.in_progress {
            bail!("TransferContext is not in progress.");
        }

        let image_subresource_range = ImageSubresourceRange::default()
            .aspect_mask(ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(mip_levels)
            .base_array_layer(0)
            .layer_count(1);

        let barrier = ImageMemoryBarrier::default()
            .old_layout(ImageLayout::UNDEFINED)
            .new_layout(ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_queue_family_index(QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(image_subresource_range)
            .src_access_mask(AccessFlags::empty())
            .dst_access_mask(AccessFlags::TRANSFER_WRITE);

        unsafe {
            self.device.cmd_pipeline_barrier(
                self.command_buffer,
                PipelineStageFlags::TOP_OF_PIPE,
                PipelineStageFlags::TRANSFER,
                DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        };

        unsafe {
            self.device.cmd_copy_buffer_to_image(
                self.command_buffer,
                self.staging_buffer.handle,
                image,
                ImageLayout::TRANSFER_DST_OPTIMAL,
                &copies,
            )
        }

        let barrier = ImageMemoryBarrier::default()
            .src_access_mask(AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(AccessFlags::empty())
            .old_layout(ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_queue_family_index(QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(image_subresource_range);

        unsafe {
            self.device.cmd_pipeline_barrier(
                self.command_buffer,
                PipelineStageFlags::TRANSFER,
                PipelineStageFlags::BOTTOM_OF_PIPE,
                DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        }

        Ok(())
    }

    pub fn align_staging(&mut self, alignment: usize) {
        self.staging_offset = (self.staging_offset + alignment - 1) & !(alignment - 1);
    }

    pub fn submit(&mut self) -> Result<()> {
        if !self.in_progress {
            bail!("TransferContext is not in progress.");
        }

        let device = &self.device;

        unsafe { device.end_command_buffer(self.command_buffer)? };

        let buffers = [self.command_buffer];
        let submit_info = SubmitInfo::default()
            .command_buffers(&buffers);

        self.queues.submit_transfer(&[submit_info], self.completion_fence)?;

        self.in_progress = false;

        Ok(())
    }

    pub fn destroy(&mut self, device_context: &DeviceContext) -> Result<()> {
        let device = &device_context.device;

        self.staging_buffer.destroy()?;

        device_context.queues.transfer_wait_idle()?;

        unsafe { device.destroy_fence(self.completion_fence, None) };

        unsafe { device.free_command_buffers(self.command_pool, &[self.command_buffer]) };
        unsafe { device.destroy_command_pool(self.command_pool, None) };

        Ok(())
    }
}
