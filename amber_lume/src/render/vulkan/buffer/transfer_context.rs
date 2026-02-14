use std::sync::Arc;
use anyhow::{Result, bail};
use ash::vk::{AccessFlags, BufferImageCopy, BufferUsageFlags, CommandPoolCreateFlags, CommandPoolCreateInfo, DependencyFlags, FenceCreateFlags, FenceCreateInfo, Image, ImageAspectFlags, ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, PipelineStageFlags, SubmitInfo, QUEUE_FAMILY_IGNORED};
use ash::{Device, vk};
use gpu_allocator::MemoryLocation;
use vk::{
    BufferCopy, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandBufferResetFlags, CommandBufferUsageFlags, CommandPool, DeviceSize,
    Fence,
};
use crate::render::vulkan::factories::buffer::linear_buffer::LinearBuffer;
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::queue::queues::Queues;

pub struct TransferContext {
    device: Device,
    queues: Arc<Queues>,

    command_pool: CommandPool,
    command_buffer: CommandBuffer,

    completion_fence: Fence,

    staging_buffer: LinearBuffer,

    in_progress: bool,
}

impl TransferContext {
    pub fn create(
        device: &Device,
        queues: Arc<Queues>,
        tag: &str,
        staging_size: DeviceSize,
        buffer_factory: &ManagedBufferFactory,
    ) -> Result<Self> {
        let staging_buffer = LinearBuffer::handle(
            buffer_factory.create_managed_buffer(
                &tag,
                staging_size,
                BufferUsageFlags::TRANSFER_SRC,
                MemoryLocation::CpuToGpu,
            )?
        );

        let command_pool = Self::create_command_pool(&device, &queues)?;

        let completion_fence_create_info = FenceCreateInfo::default()
            .flags(FenceCreateFlags::SIGNALED);
        let completion_fence = unsafe { device.create_fence(&completion_fence_create_info, None)? };

        let command_buffer_allocate_info = CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = unsafe { device.allocate_command_buffers(&command_buffer_allocate_info)?[0] };

        Ok(Self {
            device: device.clone(),
            queues: queues.clone(),

            command_pool,
            command_buffer,

            completion_fence,

            staging_buffer,

            in_progress: false,
        })
    }

    fn create_command_pool(
        device: &Device,
        queues: &Queues,
    ) -> Result<CommandPool> {
        let command_pool_create_info = CommandPoolCreateInfo::default()
            .queue_family_index(queues.transfer_queue_family())
            .flags(
                CommandPoolCreateFlags::TRANSIENT | CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            );

        let command_pool = unsafe { device.create_command_pool(&command_pool_create_info, None)? };

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

        self.staging_buffer.reset();
        self.in_progress = true;

        Ok(())
    }

    pub fn stage<T>(
        &mut self,
        data: &[T],
    ) -> Result<DeviceSize> {
        if !self.in_progress {
            bail!("TransferContext is not in progress.");
        }

        let data_size = size_of_val(data) as DeviceSize;
        let offset = self.staging_buffer.allocate_space_for(data_size)?;

        self.staging_buffer.stage(offset, data)?;

        Ok(offset)
    }

    pub fn flush_to_buffer(
        &mut self,
        target_buffer: &ManagedBuffer,
        source_offset: DeviceSize,
        target_offset: DeviceSize,
    ) -> Result<()> {
        let region = BufferCopy::default()
            .src_offset(source_offset)
            .dst_offset(target_offset)
            .size(self.staging_buffer.get_offset());

        unsafe {
            self.device.cmd_copy_buffer(
                self.command_buffer,
                self.staging_buffer.handle.handle,
                target_buffer.handle,
                &[region],
            );
        }

        Ok(())
    }

    pub fn flush_to_image(
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
                self.staging_buffer.handle.handle,
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
    
    pub fn align(&self, alignment: DeviceSize) -> Result<()> {
        self.staging_buffer.align(alignment)?;
        
        Ok(())
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

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        buffer_factory.destroy(self.staging_buffer.handle);

        unsafe { self.device.destroy_fence(self.completion_fence, None) };

        unsafe { self.device.free_command_buffers(self.command_pool, &[self.command_buffer]) };
        unsafe { self.device.destroy_command_pool(self.command_pool, None) };

        Ok(())
    }
}
