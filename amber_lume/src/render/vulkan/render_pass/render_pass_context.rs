use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::renderer::command_recording::CommandRecording;
use crate::render::vulkan::renderer::render_context::RenderContext;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use crate::snapshot_handler::world_snapshot::WorldSnapshot;
use anyhow::Result;
use ash::vk::{AccessFlags, Extent2D, ImageLayout, IndexType, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, Rect2D, RenderingInfoKHR, ShaderStageFlags, Viewport};
use bytemuck::{Pod, bytes_of};
use std::sync::Arc;
use glam::Mat4;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::vulkan::factories::buffer::pool_buffer::PoolBuffer;
use crate::render::vulkan::factories::image::swapchain_image::SwapchainImage;
use crate::render::vulkan::render_pass::ui_render_pass::ui_snapshot::UiSnapshot;
use crate::render::vulkan::render_pass::utils::transition_image_layout;

pub struct RenderView {
    pub projection_matrix: Mat4,
}

pub struct RenderViewsLayout {
    pub main: RenderView,
    pub shadows: Vec<RenderView>,
}

impl RenderViewsLayout {
    pub fn count(&self) -> u32 {
        // Main
        let mut count = 1u32;

        count += self.shadows.len() as u32;

        count
    }
}

pub struct RenderPassContext<'render_pass> {
    pub frame_index: u32,

    pub renderer_limits: &'render_pass RendererLimits,

    pub device_context: &'render_pass DeviceContext,
    pub render_context: &'render_pass RenderContext,

    pub command_recording: &'render_pass CommandRecording,

    pub swapchain_image: &'render_pass SwapchainImage,

    pub world_snapshot: Arc<WorldSnapshot>,
    
    pub ui_snapshot: UiSnapshot,

    pub render_views_layout: RenderViewsLayout,
}

impl<'render_pass> RenderPassContext<'render_pass> {
    pub fn create(
        device_context: &'render_pass DeviceContext,
        swapchain_context: &'render_pass SwapchainContext,
        render_context: &'render_pass RenderContext,
        renderer_limits: &'render_pass RendererLimits,
        command_recording: &'render_pass CommandRecording,
        image_index: u32,
        frame_index: u32,
        world_snapshot: Arc<WorldSnapshot>,
        ui_snapshot: UiSnapshot,
        render_views_layout: RenderViewsLayout,
    ) -> Result<Self> {
        let swapchain_image = swapchain_context.get_image(image_index)?;

        Ok(Self {
            frame_index,

            renderer_limits,

            device_context,
            render_context,
            
            command_recording,

            swapchain_image,

            world_snapshot,
            
            ui_snapshot,

            render_views_layout,
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

    pub fn bind_pipeline(&self, bind_point: PipelineBindPoint, pipeline: Pipeline) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe { device.cmd_bind_pipeline(command_buffer, bind_point, pipeline) };
    }
    
    pub fn bind_index_buffer(&self, buffer: &PoolBuffer) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;
        
        unsafe { device.cmd_bind_index_buffer(command_buffer, buffer.handle.handle, 0, IndexType::UINT32) };
    }

    pub fn set_viewport(&self) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;
        let extent = self.render_context.render_targets.depth_image.image_description.extent;

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
        let extent = self.render_context.render_targets.depth_image.image_description.extent;

        let scissor = Rect2D {
            offset: Offset2D { x: 0, y: 0 },
            extent: Extent2D {
                width: extent.width,
                height: extent.height,
            },
        };

        unsafe { device.cmd_set_scissor(command_buffer, 0, &[scissor]) }
    }

    pub fn push_constants<T: Pod>(
        &self,
        pipeline_layout: PipelineLayout,
        push_constants: &T,
    ) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        let slice = bytes_of(push_constants);

        unsafe {
            device.cmd_push_constants(
                command_buffer,
                pipeline_layout,
                ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT | ShaderStageFlags::COMPUTE,
                0,
                slice,
            )
        };
    }

    pub fn draw_indexed(
        &self,
        index_count: usize,
        index_offset: usize,
        vertex_offset: usize,
    ) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe {
            device.cmd_draw_indexed(
                command_buffer,
                index_count as u32,
                1,
                index_offset as u32,
                vertex_offset as i32,
                0,
            );
        }
    }

    pub fn draw_indirect_gpu_scene(
        &self,
        indirect_buffer: &PoolBuffer,
        draw_count_buffer: &PoolBuffer,
        render_view_index: u32,
    ) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe {
            device.cmd_draw_indexed_indirect_count(
                command_buffer,
                indirect_buffer.handle.handle,
                indirect_buffer.offset_to_chunk(render_view_index),
                draw_count_buffer.handle.handle,
                draw_count_buffer.offset_to_chunk(render_view_index),
                self.renderer_limits.max_draw_calls,
                indirect_buffer.item_size as u32,
            );
        }
    }

    pub fn finalize(&self) {
        transition_image_layout(
            &self,
            self.swapchain_image.image,
            self.swapchain_image.image_subresource_range,
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ImageLayout::PRESENT_SRC_KHR,
            AccessFlags::COLOR_ATTACHMENT_WRITE,
            AccessFlags::MEMORY_READ,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            PipelineStageFlags::BOTTOM_OF_PIPE,
        );
    }
}
