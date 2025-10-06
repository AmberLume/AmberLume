use anyhow::Result;
use ash::vk::{CommandBufferResetFlags, CommandBufferUsageFlags};
use ash::{Device, vk};
use std::slice;
use tracing::{debug, instrument, trace};
use vk::{
    ClearColorValue, ClearValue, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, Extent2D,
    Framebuffer, Offset2D, Rect2D, RenderPass, RenderPassBeginInfo, SubpassContents,
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

    #[instrument(level = "trace", skip_all)]
    pub fn record_pass(
        &self,
        device: &Device,
        render_pass: RenderPass,
        framebuffer: Framebuffer,
        extent: Extent2D,
    ) -> Result<()> {
        let clear_value = ClearValue {
            color: ClearColorValue {
                float32: self.clear,
            },
        };

        let render_pass_begin_info = RenderPassBeginInfo::default()
            .render_pass(render_pass)
            .framebuffer(framebuffer)
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent,
            })
            .clear_values(slice::from_ref(&clear_value));
        trace!("Begin render recording...");
        unsafe {
            device.cmd_begin_render_pass(
                self.command_buffer,
                &render_pass_begin_info,
                SubpassContents::INLINE,
            )
        }
        trace!("Render pass begun");

        trace!("Ending render pass...");
        unsafe { device.cmd_end_render_pass(self.command_buffer) }
        unsafe { device.end_command_buffer(self.command_buffer)? }
        trace!("Render pass ended");

        Ok(())
    }

    #[instrument(level = "trace", skip_all)]
    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.free_command_buffers(self.command_pool, &[self.command_buffer]);
            device.destroy_command_pool(self.command_pool, None);
        }
    }
}
