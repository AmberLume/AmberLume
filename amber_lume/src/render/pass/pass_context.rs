use crate::render::device::device_context::DeviceContext;
use crate::render::frame::command_recording::CommandRecording;
use crate::render::render_context::RenderContext;
use crate::render::swapchain::swapchain_context::SwapchainContext;
use anyhow::Result;
use ash::vk::{AccessFlags, Buffer, BufferCopy, BufferMemoryBarrier, DependencyFlags, DeviceSize, Extent2D, ImageMemoryBarrier, IndexType, MemoryBarrier, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, Rect2D, RenderingInfo, ShaderStageFlags, Viewport};
use bytemuck::{Pod, bytes_of};
use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::buffer::typed::indirect_buffer::IndirectGPU;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;
use crate::render::factories::image::swapchain_image::SwapchainImage;
use crate::ids::{FrameIndex, SliceIndex};
use crate::limits::AmberLumeLimits;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::typed_buffer::typed_buffer::TypedBuffer;
use crate::render::pass::pass_layout::RenderViewsLayout;
use crate::render::pass::ui::ui_snapshot::ClipArea;
use crate::render::render_graph::virtual_image::physical_image::PhysicalImage;
use crate::resources::resource_buffers::ResourceBuffers;
use crate::resources::skinning::bone_transform_handler::BoneTransformHandler;

pub struct PassContext<'pass> {
    pub frame_index: FrameIndex,

    pub limits: &'pass AmberLumeLimits,

    device_context: &'pass DeviceContext,
    pub render_context: &'pass RenderContext,

    pub command_recording: &'pass CommandRecording,

    pub swapchain_image: &'pass SwapchainImage,

    pub render_views_layout: &'pass RenderViewsLayout,

    pub resource_buffers: &'pass ResourceBuffers,
    pub bone_transform_handler: &'pass BoneTransformHandler,

    buffer_manager: &'pass BufferManager,
}

impl<'pass> PassContext<'pass> {
    pub fn create(
        device_context: &'pass DeviceContext,
        swapchain_context: &'pass SwapchainContext,
        render_context: &'pass RenderContext,
        limits: &'pass AmberLumeLimits,
        command_recording: &'pass CommandRecording,
        image_index: u32,
        frame_index: FrameIndex,
        render_views_layout: &'pass RenderViewsLayout,
        buffer_manager: &'pass BufferManager,
        resource_buffers: &'pass ResourceBuffers,
        bone_transform_handler: &'pass BoneTransformHandler,
    ) -> Result<Self> {
        let swapchain_image = swapchain_context.get_image(image_index)?;

        Ok(Self {
            frame_index,

            limits,

            device_context,
            render_context,
            
            command_recording,

            swapchain_image,

            render_views_layout,

            resource_buffers,
            bone_transform_handler,
            
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
    
    pub fn set_viewport(&self, physical_image: &PhysicalImage) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;
       
        let viewport = Viewport {
            x: 0.0,
            y: 0.0,
            width: physical_image.extent.width as f32,
            height: physical_image.extent.height as f32,
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
    
    pub fn set_image_scissor(&self, physical_image: &PhysicalImage) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;
      
        let scissor = Rect2D {
            offset: Offset2D { x: 0, y: 0 },
            extent: physical_image.extent,
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

    pub fn push_using_staging<'a, T>(
        &self,
        target: &BufferView<TypedBuffer<T>>,
        data: T,
        dst_access_mask: AccessFlags,
    ) -> Result<BufferMemoryBarrier<'a>> {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        let data_size = size_of_val(&data) as DeviceSize;

        let staging_buffer = &self.buffer_manager.renderer_staging_buffer;
        
        let staging_buffer_view = staging_buffer
            .frame(self.frame_index)
            .with_offset(0);
        let staging_barrier = staging_buffer_view.stage(&[data], AccessFlags::TRANSFER_READ)?;

        self.pipeline_barrier(
            PipelineStageFlags::HOST,
            PipelineStageFlags::TRANSFER,
            DependencyFlags::empty(),
            &[],
            &[
                staging_barrier,
            ],
            &[]
        );

        let src_offset = staging_buffer_view.offset();
        let dst_offset = target.offset();

        unsafe {
            device.cmd_copy_buffer(
                command_buffer,
                staging_buffer_view.handle(),
                target.get().handle(),
                &[
                    BufferCopy::default()
                        .src_offset(src_offset)
                        .dst_offset(dst_offset)
                        .size(data_size)
                ],
            );
        };

        Ok(BufferMemoryBarrier::default()
            .buffer(target.get().handle())
            .src_access_mask(AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(dst_access_mask)
            .offset(target.offset())
            .size(data_size))
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
        indirect_buffer: &BufferView<SliceBuffer<IndirectGPU>>,
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
                self.limits.resource_limits.max_draw_calls,
                indirect_buffer.item_size() as u32,
            );
        }
    }

    pub fn dispatch(&self, entity_count: u32) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        let workgroups = (entity_count + 255) / 256;

        unsafe { device.cmd_dispatch(command_buffer, workgroups, 1, 1) };
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
