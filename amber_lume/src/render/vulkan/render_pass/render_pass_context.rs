use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::image::vulkan_image::VulkanImage;
use crate::render::vulkan::renderer::command_recording::CommandRecording;
use crate::render::vulkan::renderer::render_context::RenderContext;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use anyhow::Result;
use ash::vk::RenderingInfoKHR;

pub struct RenderPassContext<'render_pass> {
    pub device_context: &'render_pass DeviceContext,
    pub swapchain_context: &'render_pass SwapchainContext,
    pub render_context: &'render_pass RenderContext,

    pub command_recording: &'render_pass CommandRecording,

    pub frame_index: usize,

    pub swapchain_image: &'render_pass VulkanImage,
}

impl<'render_pass> RenderPassContext<'render_pass> {
    pub fn create(
        device_context: &'render_pass DeviceContext,
        swapchain_context: &'render_pass SwapchainContext,
        render_context: &'render_pass RenderContext,
        command_recording: &'render_pass CommandRecording,
        frame_index: usize,
        swapchain_image_index: usize,
    ) -> Result<Self> {
        let swapchain_image = swapchain_context.get_image(swapchain_image_index)?;

        Ok(Self {
            device_context,
            swapchain_context,
            render_context,

            command_recording,

            frame_index,

            swapchain_image,
        })
    }

    pub fn begin_rendering(&self, rendering_info: &RenderingInfoKHR) {
        unsafe {
            self.render_context
                .dynamic_rendering
                .cmd_begin_rendering(self.command_recording.command_buffer, &rendering_info)
        }
    }

    pub fn end_rendering(&self) {
        unsafe {
            self.render_context
                .dynamic_rendering
                .cmd_end_rendering(self.command_recording.command_buffer)
        }
    }

    pub fn begin_command_recording(&self) -> Result<()> {
        self.command_recording
            .reset_begin_one_time(&self.device_context)
    }

    pub fn end_command_recording(&self) -> Result<()> {
        self.command_recording
            .end_command_recording(&self.device_context)
    }
}
