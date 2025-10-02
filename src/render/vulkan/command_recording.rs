use anyhow::Result;
use ash::{Device, vk};
use std::slice;
use tracing::{debug, instrument, trace};
use vk::{
    ClearColorValue, ClearValue, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, Extent2D,
    Framebuffer, Offset2D, Rect2D, RenderPass, RenderPassBeginInfo, SubpassContents,
};

pub struct CommandRecording {
    pub pool: CommandPool,
    pub buffers: Vec<CommandBuffer>,
    clear: [f32; 4],
}

impl CommandRecording {
    pub fn allocate_and_record(
        device: &Device,
        family: u32,
        render_pass: RenderPass,
        frame_buffers: &[Framebuffer],
        extent: Extent2D,
        clear: [f32; 4],
    ) -> Result<Self> {
        let command_pool_create_info = CommandPoolCreateInfo::default()
            .queue_family_index(family)
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        debug!("Creating CommandPool...");
        let command_pool = unsafe { device.create_command_pool(&command_pool_create_info, None)? };
        debug!("CommandPool created. OK");

        if frame_buffers.is_empty() {
            debug!("No FrameBuffers. Skipping CommandBuffer allocation/record");
            return Ok(Self {
                pool: command_pool,
                buffers: Vec::new(),
                clear,
            });
        }

        let command_buffer_allocate_info = CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(frame_buffers.len() as u32);
        debug!("Allocating CommandBuffers...");
        let command_buffers =
            unsafe { device.allocate_command_buffers(&command_buffer_allocate_info)? };
        debug!("CommandBuffers allocated. OK");

        Self::record_all(
            &device,
            render_pass,
            &command_buffers,
            frame_buffers,
            extent,
            clear,
        )?;

        Ok(Self {
            pool: command_pool,
            buffers: command_buffers,
            clear,
        })
    }

    #[instrument(skip_all)]
    pub fn reallocate_and_record(
        &mut self,
        device: &Device,
        render_pass: RenderPass,
        frame_buffers: &[Framebuffer],
        extent: Extent2D,
    ) -> Result<()> {
        trace!("Freeing CommandBuffers...");
        unsafe {
            device.free_command_buffers(self.pool, &self.buffers);
        }
        trace!("CommandBuffers freed. OK");

        let command_buffer_allocate_info = CommandBufferAllocateInfo::default()
            .command_pool(self.pool)
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(frame_buffers.len() as u32);
        trace!("Allocating CommandBuffers...");
        self.buffers = unsafe { device.allocate_command_buffers(&command_buffer_allocate_info)? };
        trace!("CommandBuffers allocated. OK");

        Self::record_all(
            &device,
            render_pass,
            &self.buffers,
            frame_buffers,
            extent,
            self.clear,
        )
    }

    fn record_all(
        device: &Device,
        render_pass: RenderPass,
        command_buffers: &[CommandBuffer],
        frame_buffers: &[Framebuffer],
        extent: Extent2D,
        clear: [f32; 4],
    ) -> Result<()> {
        for (command_buffer, &frame_buffer) in command_buffers.iter().zip(frame_buffers) {
            unsafe {
                debug!("Begin recording command buffer...");
                device.begin_command_buffer(*command_buffer, &CommandBufferBeginInfo::default())?;

                let clear_value = ClearValue {
                    color: ClearColorValue { float32: clear },
                };

                let render_pass_begin_info = RenderPassBeginInfo::default()
                    .render_pass(render_pass)
                    .framebuffer(frame_buffer)
                    .render_area(Rect2D {
                        offset: Offset2D { x: 0, y: 0 },
                        extent,
                    })
                    .clear_values(slice::from_ref(&clear_value));
                trace!("Begin render recording...");
                device.cmd_begin_render_pass(
                    *command_buffer,
                    &render_pass_begin_info,
                    SubpassContents::INLINE,
                );
                trace!("Render pass begun");

                trace!("Ending render pass...");
                device.cmd_end_render_pass(*command_buffer);
                device.end_command_buffer(*command_buffer)?;
                trace!("Render pass ended");
            }
        }

        Ok(())
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.free_command_buffers(self.pool, &self.buffers);
            device.destroy_command_pool(self.pool, None);
        }
    }
}
