use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, bail};
use ash::vk::{AccessFlags, Buffer, BufferImageCopy, BufferUsageFlags, CommandPoolCreateFlags, CommandPoolCreateInfo, DependencyFlags, Extent3D, FenceCreateFlags, FenceCreateInfo, Image, ImageAspectFlags, ImageLayout, ImageMemoryBarrier, ImageSubresourceLayers, ImageSubresourceRange, PipelineStageFlags, SubmitInfo, QUEUE_FAMILY_IGNORED};
use ash::{Device, vk};
use crossbeam_channel::{Receiver, Sender};
use gpu_allocator::MemoryLocation;
use vk::{
    BufferCopy, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandBufferResetFlags, CommandBufferUsageFlags, CommandPool, DeviceSize,
    Fence,
};
use crate::render::vulkan::factories::buffer::linear_buffer::LinearBuffer;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::queue::queues::Queues;

struct ImageBatch {
    regions: Vec<BufferImageCopy>,
    level_count: u32,
    layer_count: u32,
}

pub enum TransferTask {
    Buffer {
        handle: Buffer,
        data: Vec<u8>,
        offset: DeviceSize,
    },
    Image {
        handle: Image,
        data: Vec<u8>,
        extent: Extent3D,
        subresource: ImageSubresourceLayers,
        level_count: u32,
        layer_count: u32,
    },
    Terminate,
}

pub struct TransferContext {
    device: Device,
    queues: Arc<Queues>,

    command_pool: CommandPool,
    command_buffer: CommandBuffer,

    completion_fence: Fence,

    staging_buffer: LinearBuffer,

    copies_rx: Receiver<TransferTask>,
    copies_tx: Sender<TransferTask>,
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

        let (copies_tx, copies_rx) = crossbeam_channel::unbounded();

        Ok(Self {
            device: device.clone(),
            queues: queues.clone(),

            command_pool,
            command_buffer,

            completion_fence,

            staging_buffer,

            copies_rx,
            copies_tx,
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

    pub fn get_sender(&self) -> Sender<TransferTask> {
        self.copies_tx.clone()
    }

    pub fn flush(&self) -> Result<()> {
        loop {
            let mut buffer_copies: HashMap<Buffer, Vec<BufferCopy>> = HashMap::new();
            let mut buffer_image_copies: HashMap<Image, ImageBatch> = HashMap::new();

            let first_task = match self.copies_rx.recv() {
                Ok(TransferTask::Terminate) => break Ok(()),
                Ok(task) => task,
                Err(_) => break Ok(()),
            };

            self.handle_task(first_task, &mut buffer_copies, &mut buffer_image_copies)?;

            while let Ok(task) = self.copies_rx.try_recv() {
                self.handle_task(task, &mut buffer_copies, &mut buffer_image_copies)?;
            }

            if !buffer_copies.is_empty() || !buffer_image_copies.is_empty() {
                self.submit(&mut buffer_copies, &mut buffer_image_copies)?;
            }
        }
    }

    fn handle_task(
        &self,
        task: TransferTask,
        buffer_copies: &mut HashMap<Buffer, Vec<BufferCopy>>,
        buffer_image_copies: &mut HashMap<Image, ImageBatch>,
    ) -> Result<()> {
        let data_size = match &task {
            TransferTask::Buffer { data, .. } => data.len() as DeviceSize,
            TransferTask::Image { data, .. } => data.len() as DeviceSize,
            TransferTask::Terminate => unreachable!(),
        };
        let buffer_size = self.staging_buffer.handle.size;

        let buffer_space = buffer_size - self.staging_buffer.get_offset();

        if data_size > buffer_size {
            bail!("TransferContext data is too large.");
        }

        if data_size > buffer_space {
            self.submit(buffer_copies, buffer_image_copies)?;
        }

        match task {
            TransferTask::Buffer { handle, data, offset } => {
                let src_offset = self.staging_buffer.allocate_space_for(data_size)?;
                self.staging_buffer.stage(src_offset, &data)?;

                let region = BufferCopy::default()
                    .src_offset(src_offset)
                    .dst_offset(offset)
                    .size(data_size);

                buffer_copies.entry(handle).or_default().push(region);
            },
            TransferTask::Image { handle, data, extent, subresource, level_count, layer_count } => {
                let src_offset = self.staging_buffer.allocate_space_for(data_size)?;
                self.staging_buffer.stage(src_offset, &data)?;

                let region = BufferImageCopy::default()
                    .buffer_offset(src_offset)
                    .image_subresource(subresource)
                    .image_extent(extent);

                buffer_image_copies
                    .entry(handle)
                    .or_insert(ImageBatch {
                        regions: Vec::new(),
                        level_count,
                        layer_count,
                    })
                    .regions
                    .push(region);
            },
            TransferTask::Terminate => unreachable!(),
        }

        Ok(())
    }

    fn submit(
        &self,
        buffer_copies: &mut HashMap<Buffer, Vec<BufferCopy>>,
        buffer_image_copies: &mut HashMap<Image, ImageBatch>,
    ) -> Result<()> {
        unsafe {
            self.device.reset_fences(&[self.completion_fence])?;
            self.device.reset_command_buffer(self.command_buffer, CommandBufferResetFlags::empty())?;

            let begin_info = CommandBufferBeginInfo::default()
                .flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            self.device.begin_command_buffer(self.command_buffer, &begin_info)?;
        }

        self.staging_buffer.reset();

        for (dst_buffer, regions) in buffer_copies.drain() {
            unsafe {
                self.device.cmd_copy_buffer(
                    self.command_buffer,
                    self.staging_buffer.handle.handle,
                    dst_buffer,
                    &regions,
                );
            }
        }

        for (dst_image, image_batch) in buffer_image_copies.drain() {
            self.transition_layout(
                dst_image,
                image_batch.level_count,
                image_batch.layer_count,
                ImageLayout::UNDEFINED,
                ImageLayout::TRANSFER_DST_OPTIMAL,
                AccessFlags::empty(),
                AccessFlags::TRANSFER_WRITE,
                PipelineStageFlags::TOP_OF_PIPE,
                PipelineStageFlags::TRANSFER,
            );

            unsafe {
                self.device.cmd_copy_buffer_to_image(
                    self.command_buffer,
                    self.staging_buffer.handle.handle,
                    dst_image,
                    ImageLayout::TRANSFER_DST_OPTIMAL,
                    &image_batch.regions,
                );
            }

            self.transition_layout(
                dst_image,
                image_batch.level_count,
                image_batch.layer_count,
                ImageLayout::TRANSFER_DST_OPTIMAL,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::TRANSFER_WRITE,
                AccessFlags::empty(),
                PipelineStageFlags::TRANSFER,
                PipelineStageFlags::BOTTOM_OF_PIPE,
            );
        }

        unsafe { self.device.end_command_buffer(self.command_buffer)? };

        let buffers = [self.command_buffer];
        let submit_info = SubmitInfo::default()
            .command_buffers(&buffers);

        self.queues.submit_transfer(&[submit_info], self.completion_fence)?;

        Ok(())
    }

    fn transition_layout(
        &self,
        image: Image,
        level_count: u32,
        layer_count: u32,
        old_layout: ImageLayout,
        new_layout: ImageLayout,
        src_access: AccessFlags,
        dst_access: AccessFlags,
        src_stage: PipelineStageFlags,
        dst_stage: PipelineStageFlags,
    ) {
        let barrier = ImageMemoryBarrier::default()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count,
                base_array_layer: 0,
                layer_count,
            })
            .src_access_mask(src_access)
            .dst_access_mask(dst_access);

        unsafe {
            self.device.cmd_pipeline_barrier(
                self.command_buffer,
                src_stage,
                dst_stage,
                DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        };
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        drop(self.copies_rx);

        unsafe { self.device.wait_for_fences(&[self.completion_fence], true, u64::MAX)? };

        buffer_factory.destroy_buffer(self.staging_buffer.handle)?;

        unsafe { self.device.destroy_fence(self.completion_fence, None) };

        unsafe { self.device.free_command_buffers(self.command_pool, &[self.command_buffer]) };
        unsafe { self.device.destroy_command_pool(self.command_pool, None) };

        Ok(())
    }
}
