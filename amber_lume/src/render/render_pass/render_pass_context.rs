use crate::render::device::device_context::DeviceContext;
use crate::render::frame::command_recording::CommandRecording;
use crate::render::render_context::RenderContext;
use crate::render::swapchain::swapchain_context::SwapchainContext;
use crate::snapshot_handler::world_snapshot::WorldSnapshot;
use anyhow::Result;
use ash::vk::{AccessFlags, BufferCopy, DeviceSize, Extent2D, ImageLayout, IndexType, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, Rect2D, RenderingInfo, ShaderStageFlags, Viewport};
use bytemuck::{Pod, bytes_of};
use std::sync::Arc;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::buffer::typed::indirect_buffer::IndirectGpuData;
use crate::render::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;
use crate::render::factories::image::managed_image::ManagedImage;
use crate::render::factories::image::swapchain_image::SwapchainImage;
use crate::ids::{FrameIndex, SliceIndex};
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::typed_buffer::typed_buffer::TypedBuffer;
use crate::render::render_pass::render_pass_layout::RenderViewsLayout;
use crate::render::render_pass::ui::ui_snapshot::{ClipArea, UiSnapshot};
use crate::render::render_pass::utils::transition_image_layout;

pub struct RenderPassContext<'render_pass> {
    pub frame_index: FrameIndex,

    pub renderer_limits: &'render_pass RendererLimits,

    pub device_context: &'render_pass DeviceContext,
    pub render_context: &'render_pass RenderContext,

    pub command_recording: &'render_pass CommandRecording,

    pub swapchain_image: &'render_pass SwapchainImage,

    pub world_snapshot: Arc<WorldSnapshot>,
    
    pub ui_snapshot: UiSnapshot,

    pub render_views_layout: RenderViewsLayout,

    buffer_manager: &'render_pass BufferManager,
}

impl<'render_pass> RenderPassContext<'render_pass> {
    pub fn create(
        device_context: &'render_pass DeviceContext,
        swapchain_context: &'render_pass SwapchainContext,
        render_context: &'render_pass RenderContext,
        renderer_limits: &'render_pass RendererLimits,
        command_recording: &'render_pass CommandRecording,
        image_index: u32,
        frame_index: FrameIndex,
        world_snapshot: Arc<WorldSnapshot>,
        ui_snapshot: UiSnapshot,
        render_views_layout: RenderViewsLayout,
        buffer_manager: &'render_pass BufferManager,
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

            buffer_manager,
        })
    }

    pub fn begin_rendering(&self, rendering_info: &RenderingInfo) {
        unsafe {
            self.device_context
                .device
                .cmd_begin_rendering(self.command_recording.command_buffer, &rendering_info)
        }
    }

    pub fn end_rendering(&self) {
        unsafe {
            self.device_context
                .device
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

    pub fn bind_index_buffer(&self, buffer_view: BufferView<SliceBuffer<u32>>) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        let buffer_start = buffer_view.slice_at(SliceIndex::ZERO);
        
        unsafe { device.cmd_bind_index_buffer(command_buffer, buffer_start.handle(), buffer_start.offset(), IndexType::UINT32) };
    }

    pub fn clear_buffer<T: BufferInfo>(&self, buffer: &T) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe {
            device.cmd_fill_buffer(
                command_buffer,
                buffer.handle(),
                0,
                buffer.entire_size(),
                0,
            )
        };
    }
    
    pub fn set_viewport(&self, managed_image: &ManagedImage) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;
        let extent = managed_image.image_description.extent;

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

    pub fn set_area_scissor(&self, clip_area: &ClipArea) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        let scissor = Rect2D {
            offset: Offset2D { 
                x: clip_area.position[0], 
                y: clip_area.position[1],
            },
            extent: Extent2D {
                width: clip_area.size[0],
                height: clip_area.size[1],
            },
        };

        unsafe { device.cmd_set_scissor(command_buffer, 0, &[scissor]) }
    }
    
    pub fn set_image_scissor(&self, managed_image: &ManagedImage) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;
        let extent = managed_image.image_description.extent;

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

    pub fn push_using_staging<T>(
        &self,
        target: &BufferView<ManagedBuffer>,
        data: T,
    ) -> Result<()> {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        let data_size = size_of_val(&data) as DeviceSize;
        
        let staging_buffer = &self.buffer_manager.renderer_staging_buffer;
        
        let staging_buffer_view = staging_buffer
            .frame(self.frame_index)
            .offset(0);
        staging_buffer_view.stage(&[data])?;

        let src_offset = staging_buffer_view.offset();
        let dst_offset = target.offset();

        unsafe {
            device.cmd_copy_buffer(
                command_buffer,
                staging_buffer_view.handle(),
                target.handle(),
                &[
                    BufferCopy::default()
                        .src_offset(src_offset)
                        .dst_offset(dst_offset)
                        .size(data_size)
                ],
            );
        };

        Ok(())
    }

    pub fn draw(
        &self,
        vertex_count: u32,
    ) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe {
            device.cmd_draw(
                command_buffer,
                vertex_count,
                1,
                0,
                0,
            );
        }
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
        indirect_buffer: &BufferView<SliceBuffer<IndirectGpuData>>,
        draw_count_buffer: &BufferView<TypedBuffer<u32>>,
    ) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        let indirect_buffer_view = indirect_buffer.slice_at(SliceIndex::ZERO);
        let draw_count_buffer_view = draw_count_buffer.get();

        unsafe {
            device.cmd_draw_indexed_indirect_count(
                command_buffer,
                indirect_buffer_view.handle(),
                indirect_buffer_view.offset(),
                draw_count_buffer_view.handle(),
                draw_count_buffer_view.offset(),
                self.renderer_limits.render_resource_limits.max_draw_calls,
                indirect_buffer.item_size() as u32,
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
