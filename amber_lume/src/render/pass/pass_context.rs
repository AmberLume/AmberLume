use crate::render::device::device_context::DeviceContext;
use crate::render::frame::command_recording::CommandRecording;
use crate::render::render_context::RenderContext;
use anyhow::Result;
use ash::vk::{AccessFlags, Buffer, BufferImageCopy, BufferMemoryBarrier, ClearColorValue, ClearDepthStencilValue, DependencyFlags, DeviceSize, Extent2D, Extent3D, Image, ImageAspectFlags, ImageLayout, ImageMemoryBarrier, ImageSubresourceLayers, ImageSubresourceRange, IndexType, MemoryBarrier, Offset2D, Offset3D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, Rect2D, RenderingInfo, ShaderStageFlags, Viewport};
use bytemuck::{Pod, bytes_of};
use crate::render::buffer::typed::indirect_buffer::IndirectGPU;
use crate::render::factories::buffer::view::buffer_view::BufferView;
use crate::ids::FrameIndex;
use crate::limits::AmberLumeLimits;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::pass::pass_layout::RenderViewsLayout;
use crate::render::pass::ui::ui_snapshot::ClipArea;
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;
use crate::render::render_graph::virtual_image::physical_image::PhysicalImage;
use crate::render::target::render_target::RenderTargetImage;
use crate::resources::resource_buffers::ResourceBuffers;

pub struct PassContext<'pass> {
    pub frame_index: FrameIndex,
    pub frame_number: u32,

    pub history_write_index: u32,
    pub history_valid: bool,

    pub limits: &'pass AmberLumeLimits,

    device_context: &'pass DeviceContext,
    pub render_context: &'pass RenderContext,

    pub command_recording: &'pass CommandRecording,

    pub render_target_image: RenderTargetImage,

    pub render_views_layout: &'pass RenderViewsLayout,

    pub resource_buffers: &'pass ResourceBuffers,
}

impl<'pass> PassContext<'pass> {
    pub fn create(
        device_context: &'pass DeviceContext,
        render_context: &'pass RenderContext,
        limits: &'pass AmberLumeLimits,
        command_recording: &'pass CommandRecording,
        render_target_image: RenderTargetImage,
        frame_index: FrameIndex,
        frame_number: u32,
        history_write_index: u32,
        history_valid: bool,
        render_views_layout: &'pass RenderViewsLayout,
        resource_buffers: &'pass ResourceBuffers,
    ) -> Result<Self> {
        Ok(Self {
            frame_index,
            frame_number,

            history_write_index,
            history_valid,

            limits,

            device_context,
            render_context,

            command_recording,

            render_target_image,

            render_views_layout,

            resource_buffers,
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

    pub fn bind_index_buffer(&self) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe { device.cmd_bind_index_buffer(command_buffer, self.resource_buffers.index_buffer_handle, 0, IndexType::UINT32) };
    }

    pub fn bind_ui_index_buffer(&self, handle: Buffer, offset: DeviceSize) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe { device.cmd_bind_index_buffer(command_buffer, handle, offset, IndexType::UINT32) };
    }

    pub fn clear_buffer<'a, T: BufferInfo>(
        &self, 
        buffer_view: BufferView<T>,
        size: DeviceSize,
        dst_access_mask: AccessFlags,
    ) -> BufferMemoryBarrier<'a> {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        let handle = buffer_view.inner().handle();
        let offset = buffer_view.offset();
        
        unsafe {
            device.cmd_fill_buffer(
                command_buffer,
                handle,
                offset,
                size,
                0,
            )
        };

        BufferMemoryBarrier::default()
            .buffer(handle)
            .src_access_mask(AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(dst_access_mask)
            .offset(offset)
            .size(size)
    }

    pub fn clear_buffer_raw<'a>(
        &self,
        buffer: Buffer,
        offset: DeviceSize,
        size: DeviceSize,
        dst_access_mask: AccessFlags,
    ) -> BufferMemoryBarrier<'a> {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe {
            device.cmd_fill_buffer(
                command_buffer,
                buffer,
                offset,
                size,
                0,
            )
        };

        BufferMemoryBarrier::default()
            .buffer(buffer)
            .src_access_mask(AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(dst_access_mask)
            .offset(offset)
            .size(size)
    }

    pub fn clear_depth_stencil_image(&self, image: Image, subresource_range: ImageSubresourceRange, depth: f32) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe {
            device.cmd_clear_depth_stencil_image(
                command_buffer,
                image,
                ImageLayout::TRANSFER_DST_OPTIMAL,
                &ClearDepthStencilValue { depth, stencil: 0 },
                &[subresource_range],
            );
        }
    }

    pub fn copy_image_texel_to_buffer(&self, image: Image, x: i32, y: i32, buffer: Buffer, buffer_offset: DeviceSize) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        let region = BufferImageCopy::default()
            .buffer_offset(buffer_offset)
            .image_subresource(
                ImageSubresourceLayers::default()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .image_offset(Offset3D { x, y, z: 0 })
            .image_extent(Extent3D { width: 1, height: 1, depth: 1 });

        unsafe {
            device.cmd_copy_image_to_buffer(
                command_buffer,
                image,
                ImageLayout::TRANSFER_SRC_OPTIMAL,
                buffer,
                &[region],
            );
        }
    }

    pub fn clear_color_image(&self, image: Image, subresource_range: ImageSubresourceRange, color: [f32; 4]) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe {
            device.cmd_clear_color_image(
                command_buffer,
                image,
                ImageLayout::TRANSFER_DST_OPTIMAL,
                &ClearColorValue { float32: color },
                &[subresource_range],
            );
        }
    }

    pub fn set_render_area(&self, extent: Extent2D) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe {
            device.cmd_set_viewport(command_buffer, 0, &[Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }]);
            device.cmd_set_scissor(command_buffer, 0, &[Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent,
            }]);
        }
    }

    pub fn set_area_scissor(&self, clip_area: &ClipArea) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe {
            device.cmd_set_scissor(command_buffer, 0, &[Rect2D {
                offset: Offset2D {
                    x: clip_area.position[0],
                    y: clip_area.position[1],
                },
                extent: Extent2D {
                    width: clip_area.size[0],
                    height: clip_area.size[1],
                },
            }])
        }
    }
    
    pub fn set_image_scissor(&self, physical_image: &PhysicalImage) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;
      
        unsafe { device.cmd_set_scissor(command_buffer, 0, &[Rect2D {
            offset: Offset2D { x: 0, y: 0 },
            extent: physical_image.extent,
        }]) }
    }

    pub fn push_constants<T: Pod>(
        &self,
        pipeline_layout: PipelineLayout,
        push_constants: &T,
    ) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe {
            device.cmd_push_constants(
                command_buffer,
                pipeline_layout,
                ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT | ShaderStageFlags::COMPUTE,
                0,
                bytes_of(push_constants),
            )
        };
    }

    pub fn draw(&self, vertex_count: u32) {
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
        indirect_buffer: &PhysicalBuffer,
        draw_count_buffer: &PhysicalBuffer,
    ) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe {
            device.cmd_draw_indexed_indirect_count(
                command_buffer,
                indirect_buffer.buffer,
                indirect_buffer.offset,
                draw_count_buffer.buffer,
                draw_count_buffer.offset,
                self.limits.resource_limits.max_draw_calls,
                size_of::<IndirectGPU>() as u32,
            );
        }
    }

    pub fn dispatch(&self, entity_count: u32) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        let workgroups = (entity_count + 255) / 256;

        unsafe { device.cmd_dispatch(command_buffer, workgroups, 1, 1) };
    }

    pub fn dispatch_2d(&self, width: u32, height: u32) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        let groups_x = (width + 15) / 16;
        let groups_y = (height + 15) / 16;

        unsafe { device.cmd_dispatch(command_buffer, groups_x, groups_y, 1) };
    }

    pub fn pipeline_barrier(
        &self,
        src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags,
        dependency_flags: DependencyFlags,
        memory_barriers: &[MemoryBarrier],
        buffer_memory_barriers: &[BufferMemoryBarrier],
        image_memory_barriers: &[ImageMemoryBarrier],
    ) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                src_stage_mask,
                dst_stage_mask,
                dependency_flags,
                memory_barriers,
                buffer_memory_barriers,
                image_memory_barriers,
            )
        }
    }
}
