use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::image::vulkan_image::VulkanImage;
use crate::render::vulkan::renderer::command_recording::CommandRecording;
use crate::render::vulkan::renderer::render_context::RenderContext;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use anyhow::Result;
use ash::vk::{
    Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, Rect2D, RenderingInfoKHR,
    ShaderStageFlags, Viewport,
};
use std::slice::from_raw_parts;

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

    pub fn bind_pipeline(&self, pipeline: Pipeline) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe { device.cmd_bind_pipeline(command_buffer, PipelineBindPoint::GRAPHICS, pipeline) };
    }

    pub fn set_viewport(&self) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;
        let extent = self.render_context.render_targets.depth_vulkan_image.extent;

        let viewport = Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        unsafe { device.cmd_set_viewport(command_buffer, 0, &[viewport]) }
    }

    pub fn set_scissor(&self) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;
        let extent = self.render_context.render_targets.depth_vulkan_image.extent;

        let scissor = Rect2D {
            offset: Offset2D { x: 0, y: 0 },
            extent,
        };

        unsafe { device.cmd_set_scissor(command_buffer, 0, &[scissor]) }
    }

    pub fn push_constants<T>(
        &self,
        pipeline_layout: PipelineLayout,
        offset: u32,
        stage: ShaderStageFlags,
        push_constants: &T,
    ) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe {
            device.cmd_push_constants(
                command_buffer,
                pipeline_layout,
                stage,
                offset,
                from_raw_parts(&push_constants as *const _ as *const u8, size_of::<T>()),
            )
        };
    }
}
