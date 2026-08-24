use gpu::{BufferRange, PipelineLayoutFactory};
use std::mem::size_of;
use ash::vk::{AccessFlags, Buffer, BufferMemoryBarrier, ClearColorValue, ClearDepthStencilValue, CommandBuffer, DependencyFlags, DeviceSize, Extent2D, Image, ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, IndexType, MemoryBarrier, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, Rect2D, RenderingInfo, ShaderStageFlags, Viewport};
use bytemuck::{Pod, bytes_of};
use crate::indirect_gpu::IndirectGPU;
use crate::draw_bucket::DrawBucket;
use index_allocator::FrameIndex;
use crate::virtual_buffer::physical_buffer::PhysicalBuffer;
use anyhow::Result;
use ash::Device;
use gpu::CommandRecording;
use gpu::DeviceContext;
use gpu::RenderTargetImage;

pub struct FrameContext<'pass> {
    device_context: &'pass DeviceContext,
    command_recording: &'pass CommandRecording,

    pub render_target_image: RenderTargetImage,

    pub frame_index: FrameIndex,
    pub frame_number: u32,

    pub history_write_index: u32,
    pub history_valid: bool,
}

impl<'pass> FrameContext<'pass> {
    pub fn create(
        device_context: &'pass DeviceContext,
        command_recording: &'pass CommandRecording,
        render_target_image: RenderTargetImage,
        frame_index: FrameIndex,
        frame_number: u32,
        history_write_index: u32,
        history_valid: bool,
    ) -> Self {
        Self {
            device_context,
            command_recording,

            render_target_image,

            frame_index,
            frame_number,

            history_write_index,
            history_valid,
        }
    }

    pub fn device(&self) -> &Device {
        &self.device_context.device
    }

    pub fn command_buffer(&self) -> CommandBuffer {
        self.command_recording.command_buffer
    }

    pub fn begin_command_recording(&self) -> Result<()> {
        self.command_recording.reset_begin_one_time(self.device_context)
    }

    pub fn end_command_recording(&self) -> Result<()> {
        self.command_recording.end_command_recording(self.device_context)
    }

    pub fn begin_rendering(&self, rendering_info: &RenderingInfo) {
        unsafe { self.device().cmd_begin_rendering(self.command_buffer(), rendering_info) }
    }

    pub fn end_rendering(&self) {
        unsafe { self.device().cmd_end_rendering(self.command_buffer()) }
    }

    pub fn set_render_area(&self, extent: Extent2D) {
        let device = self.device();
        let command_buffer = self.command_buffer();

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

    pub fn clear_color_image(&self, image: Image, subresource_range: ImageSubresourceRange, color: [f32; 4]) {
        unsafe {
            self.device().cmd_clear_color_image(
                self.command_buffer(),
                image,
                ImageLayout::TRANSFER_DST_OPTIMAL,
                &ClearColorValue { float32: color },
                &[subresource_range],
            );
        }
    }

    pub fn clear_depth_stencil_image(&self, image: Image, subresource_range: ImageSubresourceRange, depth: f32) {
        unsafe {
            self.device().cmd_clear_depth_stencil_image(
                self.command_buffer(),
                image,
                ImageLayout::TRANSFER_DST_OPTIMAL,
                &ClearDepthStencilValue { depth, stencil: 0 },
                &[subresource_range],
            );
        }
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
        unsafe {
            self.device().cmd_pipeline_barrier(
                self.command_buffer(),
                src_stage_mask,
                dst_stage_mask,
                dependency_flags,
                memory_barriers,
                buffer_memory_barriers,
                image_memory_barriers,
            )
        }
    }

    pub fn bind_pipeline(&self, bind_point: PipelineBindPoint, pipeline: Pipeline) {
        let device = self.device();
        let command_buffer = self.command_buffer();

        unsafe { device.cmd_bind_pipeline(command_buffer, bind_point, pipeline) };
    }

    pub fn bind_index_buffer(&self, buffer_range: BufferRange, offset: DeviceSize) {
        let device = self.device();
        let command_buffer = self.command_buffer();

        unsafe { device.cmd_bind_index_buffer(command_buffer, buffer_range.buffer, offset, IndexType::UINT32) };
    }

    pub fn fill_buffer(&self, buffer_range: BufferRange, value: u32) {
        unsafe {
            self.device().cmd_fill_buffer(
                self.command_buffer(),
                buffer_range.buffer,
                buffer_range.offset,
                buffer_range.size,
                value,
            )
        };
    }

    pub fn clear_buffer_raw<'a>(
        &self,
        buffer: Buffer,
        offset: DeviceSize,
        size: DeviceSize,
        value: u32,
        dst_access_mask: AccessFlags,
    ) -> BufferMemoryBarrier<'a> {
        let device = self.device();
        let command_buffer = self.command_buffer();

        unsafe {
            device.cmd_fill_buffer(
                command_buffer,
                buffer,
                offset,
                size,
                value,
            )
        };

        BufferMemoryBarrier::default()
            .buffer(buffer)
            .src_access_mask(AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(dst_access_mask)
            .offset(offset)
            .size(size)
    }

    pub fn set_scissor(&self, offset: Offset2D, extent: Extent2D) {
        let device = self.device();
        let command_buffer = self.command_buffer();

        unsafe { device.cmd_set_scissor(command_buffer, 0, &[Rect2D { offset, extent }]) }
    }

    pub fn push_constants<T: Pod>(
        &self,
        pipeline_layout: PipelineLayout,
        push_constants: &T,
    ) {
        const {
            assert!(
                size_of::<T>() <= PipelineLayoutFactory::PUSH_CONSTANTS_SIZE as usize,
                "Push constants exceed the pipeline layout budget",
            );
        }

        let device = self.device();
        let command_buffer = self.command_buffer();

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
        let device = self.device();
        let command_buffer = self.command_buffer();

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
        let device = self.device();
        let command_buffer = self.command_buffer();

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
        bucket: DrawBucket,
    ) {
        let device = self.device();
        let command_buffer = self.command_buffer();

        unsafe {
            device.cmd_draw_indexed_indirect_count(
                command_buffer,
                indirect_buffer.range.buffer,
                indirect_buffer.range.offset + bucket.draw_offset as DeviceSize * size_of::<IndirectGPU>() as DeviceSize,
                draw_count_buffer.range.buffer,
                draw_count_buffer.range.offset + bucket.count_index as DeviceSize * size_of::<u32>() as DeviceSize,
                bucket.capacity,
                size_of::<IndirectGPU>() as u32,
            );
        }
    }

    pub fn dispatch(&self, entity_count: u32) {
        let device = self.device();
        let command_buffer = self.command_buffer();

        let workgroups = (entity_count + 255) / 256;

        unsafe { device.cmd_dispatch(command_buffer, workgroups, 1, 1) };
    }

    pub fn dispatch_2d(&self, width: u32, height: u32) {
        let device = self.device();
        let command_buffer = self.command_buffer();

        let groups_x = (width + 15) / 16;
        let groups_y = (height + 15) / 16;

        unsafe { device.cmd_dispatch(command_buffer, groups_x, groups_y, 1) };
    }

    pub fn dispatch_groups(&self, groups_x: u32, groups_y: u32, groups_z: u32) {
        let device = self.device();
        let command_buffer = self.command_buffer();

        unsafe { device.cmd_dispatch(command_buffer, groups_x, groups_y, groups_z) };
    }
}
