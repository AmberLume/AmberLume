use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::render_pass::render_pass::RenderPass;
use crate::render::vulkan::render_pass::render_pass_context::RenderPassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, BufferMemoryBarrier, DependencyFlags, DeviceAddress, MemoryBarrier, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, WHOLE_SIZE};
use std::sync::Arc;
use bytemuck::bytes_of;
use tracing::info;
use crate::render::vulkan::resource_context::ResourceContext;
use crate::render::vulkan::buffer::typed::entity_buffer::EntityGpuData;
use crate::render::vulkan::buffer::typed::scene_buffer::SceneGpuData;
use crate::render::vulkan::render_pass::culling_pass::culling_push_constants::{CullingPushConstants, FrustumPlanes};
use crate::render::vulkan::renderer::stats::gpu_render_stats::GpuRenderStats;
use crate::render::vulkan::renderer::stats::gpu_render_stats_handler::GpuRenderStatsHandler;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::persistent::persistent_resources::PersistentResources;

pub struct CullingRenderPass {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    _compute_pipeline_handle: Arc<ResRef>,

    gpu_render_stats_buffer_device_address: DeviceAddress,
    
    buffer_manager: Arc<BufferManager>,
}

impl CullingRenderPass {
    pub fn create(
        resource_context: &ResourceContext,
        compute_pipeline_provider: &ResourceProvider<ComputePipelineBackend>,
        persistent_resources: &PersistentResources,
        render_stats_reader: &GpuRenderStatsHandler,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: String::from("shaders/culling.comp.spv"),
            fn_name: String::from("main"),
        };

        let compute_pipeline_handle = compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = compute_pipeline_provider.get_resource(compute_pipeline_handle.id) else {
            bail!("Failed to acquire ComputePipeline");
        };

        Ok(Self {
            pipeline,
            pipeline_layout: persistent_resources.pipeline_layouts.global,

            _compute_pipeline_handle: compute_pipeline_handle,
            
            gpu_render_stats_buffer_device_address: render_stats_reader.buffer.device_address.unwrap(),
            
            buffer_manager: resource_context.buffer_manager.clone(),
        })
    }
}

impl RenderPass for CullingRenderPass {
    fn is_enabled(&self) -> bool {
        true
    }

    fn begin_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        let device = &render_pass_context.device_context.device;
        let command_buffer = render_pass_context.command_recording.command_buffer;

        let extent = render_pass_context.swapchain_image.extent;
        let aspect_ratio = extent.width as f32 / extent.height as f32;

        let scene_gpu_data = SceneGpuData::create(
            render_pass_context.world_snapshot.camera_stamp.to_view_projection_matrix(aspect_ratio),
            &self.buffer_manager,
        );
        self.buffer_manager.scene_buffer.stage(0, &bytes_of(&scene_gpu_data))?;

        unsafe { 
            device.cmd_fill_buffer(
                command_buffer, 
                self.buffer_manager.draw_count_buffer.handle.handle, 
                0, 
                self.buffer_manager.draw_count_buffer.handle.size,
                0,
            ) 
        };

        let entities_gpu_data: Vec<EntityGpuData> = render_pass_context.world_snapshot.entities.iter().map(|entity| {
            EntityGpuData::create(entity.transform_matrix, entity.model_id)
        }).collect();
        self.buffer_manager.entity_buffer.replace_with(&entities_gpu_data)?;

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                PipelineStageFlags::TRANSFER,
                PipelineStageFlags::COMPUTE_SHADER,
                DependencyFlags::empty(),
                &[
                    MemoryBarrier::default()
                        .src_access_mask(AccessFlags::TRANSFER_WRITE)
                        .dst_access_mask(AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE)
                ],
                &[],
                &[],
            )
        };

        Ok(())
    }

    fn record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        let device = &render_pass_context.device_context.device;
        let command_buffer = render_pass_context.command_recording.command_buffer;

        render_pass_context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        let entity_count = render_pass_context.world_snapshot.entities.len() as u32;
        if entity_count == 0 {
            return Ok(());
        }

        let stats_size = size_of::<GpuRenderStats>() as DeviceAddress;
        let stats_buffer_device_address_offset = stats_size * render_pass_context.frame_index as DeviceAddress;

        let extent = render_pass_context.swapchain_image.extent;
        let aspect_ratio = extent.width as f32 / extent.height as f32;

        let projection_matrix = render_pass_context.world_snapshot.camera_stamp.to_view_projection_matrix(aspect_ratio);
        let frustum_planes = FrustumPlanes::from_matrix(projection_matrix);

        render_pass_context.push_constants(
            self.pipeline_layout,
            &CullingPushConstants::create(
                self.buffer_manager.scene_buffer.handle.device_address.unwrap(),
                self.gpu_render_stats_buffer_device_address + stats_buffer_device_address_offset,
                frustum_planes,
                entity_count,
            ),
        );

        let workgroups = (entity_count + 255) / 256;

        unsafe { device.cmd_dispatch(command_buffer, workgroups, 1, 1) };

        Ok(())
    }

    fn end_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        let device = &render_pass_context.device_context.device;
        let command_buffer = render_pass_context.command_recording.command_buffer;

        let barriers = [
            BufferMemoryBarrier::default()
                .src_access_mask(AccessFlags::SHADER_WRITE)
                .dst_access_mask(AccessFlags::INDIRECT_COMMAND_READ)
                .buffer(self.buffer_manager.draw_count_buffer.handle.handle)
                .size(WHOLE_SIZE),
            BufferMemoryBarrier::default()
                .src_access_mask(AccessFlags::SHADER_WRITE)
                .dst_access_mask(AccessFlags::INDIRECT_COMMAND_READ)
                .buffer(self.buffer_manager.indirect_buffer.handle.handle)
                .size(WHOLE_SIZE),
            BufferMemoryBarrier::default()
                .src_access_mask(AccessFlags::SHADER_WRITE)
                .dst_access_mask(AccessFlags::SHADER_READ)
                .buffer(self.buffer_manager.draw_buffer.handle.handle)
                .size(WHOLE_SIZE),
        ];

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                PipelineStageFlags::COMPUTE_SHADER,
                PipelineStageFlags::DRAW_INDIRECT | PipelineStageFlags::VERTEX_SHADER,
                DependencyFlags::empty(),
                &[],
                &barriers,
                &[],
            );
        };

        Ok(())
    }

    fn destroy(&self) -> Result<()> {
        info!("CullingRenderPass destroyed");

        Ok(())
    }
}
