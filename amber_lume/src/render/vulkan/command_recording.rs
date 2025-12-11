use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::khr::dynamic_rendering;
use ash::vk::{
    AccessFlags, AttachmentLoadOp, AttachmentStoreOp, ClearDepthStencilValue,
    CommandBufferResetFlags, CommandBufferUsageFlags, DependencyFlags, Format, Image,
    ImageAspectFlags, ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, ImageView,
    PipelineStageFlags, QUEUE_FAMILY_IGNORED, RenderingAttachmentInfoKHR, RenderingInfoKHR,
};
use ash::{Device, vk};
use tracing::{debug, info, instrument};
use vk::{
    ClearColorValue, ClearValue, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, Extent2D,
    Offset2D, Rect2D,
};

pub struct CommandRecording {
    pub command_pool: CommandPool,
    pub command_buffer: CommandBuffer,

    pub clear: [f32; 4],
}

impl CommandRecording {
    pub fn create(device: &Device, queue_family: u32, clear: [f32; 4]) -> Result<Self> {
        let command_pool_create_info = CommandPoolCreateInfo::default()
            .queue_family_index(queue_family)
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        debug!("Creating CommandPool...");
        let command_pool = unsafe { device.create_command_pool(&command_pool_create_info, None)? };
        debug!("CommandPool created. OK");

        let command_buffer_allocate_info = CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        debug!("Allocating CommandBuffers...");
        let command_buffer =
            unsafe { device.allocate_command_buffers(&command_buffer_allocate_info)?[0] };
        debug!("CommandBuffers allocated. OK");

        Ok(Self {
            command_pool,
            command_buffer,

            clear,
        })
    }

    #[instrument(level = "trace", skip_all)]
    pub fn reset_begin_one_time(&self, device: &Device) -> Result<()> {
        unsafe {
            device.reset_command_buffer(self.command_buffer, CommandBufferResetFlags::empty())?
        }
        let command_buffer_begin_info =
            CommandBufferBeginInfo::default().flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { device.begin_command_buffer(self.command_buffer, &command_buffer_begin_info)? }

        Ok(())
    }

    pub fn transition_image_layout(
        &self,
        device: &Device,
        image: Image,
        aspect_mask: ImageAspectFlags,
        old_layout: ImageLayout,
        new_layout: ImageLayout,
        src_access: AccessFlags,
        dst_access: AccessFlags,
        src_stage: PipelineStageFlags,
        dst_stage: PipelineStageFlags,
    ) {
        let subresource_range = ImageSubresourceRange::default()
            .aspect_mask(aspect_mask)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let barrier = ImageMemoryBarrier::default()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(subresource_range)
            .src_access_mask(src_access)
            .dst_access_mask(dst_access);

        unsafe {
            device.cmd_pipeline_barrier(
                self.command_buffer,
                src_stage,
                dst_stage,
                DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        }
    }

    #[instrument(level = "trace", skip_all)]
    pub fn record_pass(
        &self,
        device_context: &DeviceContext,
        dynamic_rendering: &dynamic_rendering::Device,
        swapchain_image: Image,
        color_image_view: ImageView,
        depth_image: Image,
        depth_image_view: ImageView,
        depth_format: Format,
        extent: Extent2D,
    ) -> Result<()> {
        self.transition_image_layout(
            &device_context.device,
            swapchain_image,
            ImageAspectFlags::COLOR,
            ImageLayout::UNDEFINED,
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            AccessFlags::empty(),
            AccessFlags::COLOR_ATTACHMENT_WRITE,
            PipelineStageFlags::TOP_OF_PIPE,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        );

        let depth_aspect = Self::get_depth_aspect_mask(depth_format);

        self.transition_image_layout(
            &device_context.device,
            depth_image,
            depth_aspect,
            ImageLayout::UNDEFINED,
            ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            AccessFlags::empty(),
            AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                | AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            PipelineStageFlags::TOP_OF_PIPE,
            PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
        );

        let color_attachment = RenderingAttachmentInfoKHR::default()
            .image_view(color_image_view)
            .image_layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .clear_value(ClearValue {
                color: ClearColorValue {
                    float32: self.clear,
                },
            });

        let depth_attachment = RenderingAttachmentInfoKHR::default()
            .image_view(depth_image_view)
            .image_layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .clear_value(ClearValue {
                depth_stencil: ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            });

        let color_attachments = [color_attachment];

        let rendering_info = RenderingInfoKHR::default()
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent,
            })
            .layer_count(1)
            .color_attachments(&color_attachments)
            .depth_attachment(&depth_attachment);

        unsafe { dynamic_rendering.cmd_begin_rendering(self.command_buffer, &rendering_info) };
        unsafe { dynamic_rendering.cmd_end_rendering(self.command_buffer) };

        self.transition_image_layout(
            &device_context.device,
            swapchain_image,
            ImageAspectFlags::COLOR,
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ImageLayout::PRESENT_SRC_KHR,
            AccessFlags::COLOR_ATTACHMENT_WRITE,
            AccessFlags::empty(),
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            PipelineStageFlags::BOTTOM_OF_PIPE,
        );

        unsafe {
            device_context
                .device
                .end_command_buffer(self.command_buffer)?
        }

        Ok(())
    }

    fn get_depth_aspect_mask(format: Format) -> ImageAspectFlags {
        match format {
            Format::D16_UNORM | Format::D32_SFLOAT | Format::X8_D24_UNORM_PACK32 => {
                ImageAspectFlags::DEPTH
            }
            Format::D16_UNORM_S8_UINT | Format::D24_UNORM_S8_UINT | Format::D32_SFLOAT_S8_UINT => {
                ImageAspectFlags::DEPTH | ImageAspectFlags::STENCIL
            }
            Format::S8_UINT => ImageAspectFlags::STENCIL,
            _ => ImageAspectFlags::DEPTH,
        }
    }

    pub fn destroy(&self, device_context: &DeviceContext) -> Result<()> {
        let device = &device_context.device;

        unsafe { device.free_command_buffers(self.command_pool, &[self.command_buffer]) };
        unsafe { device.destroy_command_pool(self.command_pool, None) };

        info!("CommandRecording destroyed");

        Ok(())
    }
}
