use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::render_pass::render_pass::RenderPass;
use crate::render::vulkan::render_pass::render_pass_context::RenderPassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, BufferMemoryBarrier, DependencyFlags, MemoryBarrier, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, WHOLE_SIZE};
use std::sync::Arc;
use tracing::info;
use crate::render::vulkan::buffer::typed::collider_buffer::ColliderGpuData;
use crate::render::vulkan::resource_context::ResourceContext;
use crate::render::vulkan::render_pass::collider_culling_pass::collider_culling_push_constants::ColliderCullingPushConstants;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::persistent::persistent_resources::PersistentResources;

pub struct ColliderCullingRenderPass {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    _compute_pipeline_handle: Arc<ResRef>,

    buffer_manager: Arc<BufferManager>,
}

impl ColliderCullingRenderPass {
    pub fn create(
        resource_context: &ResourceContext,
        compute_pipeline_provider: &ResourceProvider<ComputePipelineBackend>,
        persistent_resources: &PersistentResources,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: String::from("shaders/collider_culling.comp.spv"),
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

            buffer_manager: resource_context.buffer_manager.clone(),
        })
    }
}

impl RenderPass for ColliderCullingRenderPass {
    fn is_enabled(&self) -> bool {
        true
    }

    fn begin_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        let device = &render_pass_context.device_context.device;
        let command_buffer = render_pass_context.command_recording.command_buffer;

        let colliders_gpu_data: Vec<ColliderGpuData> = render_pass_context.world_snapshot.colliders.iter().map(|collider| {
            ColliderGpuData::create(collider.transform_matrix, collider.half_extents, collider.color, collider.shape_type)
        }).collect();
        self.buffer_manager.collider_buffer.replace_with(&colliders_gpu_data)?;

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

        let collider_count = render_pass_context.world_snapshot.colliders.len() as u32;
        if collider_count == 0 {
            return Ok(());
        }

        render_pass_context.push_constants(
            self.pipeline_layout,
            &ColliderCullingPushConstants::create(
                self.buffer_manager.collider_indirect_buffer.handle.device_address.unwrap(),
                self.buffer_manager.draw_count_buffer.handle.device_address.unwrap(),
                self.buffer_manager.collider_buffer.handle.device_address.unwrap(),
                collider_count,
            ),
        );

        let workgroups = (collider_count + 255) / 256;

        unsafe { device.cmd_dispatch(command_buffer, workgroups, 1, 1) };

        Ok(())
    }

    fn end_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        let device = &render_pass_context.device_context.device;
        let command_buffer = render_pass_context.command_recording.command_buffer;

        let buffer_barrier = BufferMemoryBarrier::default()
            .src_access_mask(AccessFlags::SHADER_WRITE)
            .dst_access_mask(AccessFlags::INDIRECT_COMMAND_READ)
            .buffer(self.buffer_manager.collider_indirect_buffer.handle.handle)
            .size(WHOLE_SIZE);
        
        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                PipelineStageFlags::COMPUTE_SHADER,
                PipelineStageFlags::DRAW_INDIRECT,
                DependencyFlags::empty(),
                &[],
                &[buffer_barrier],
                &[],
            );
        };

        Ok(())
    }

    fn destroy(&self) -> Result<()> {
        info!("ColliderCullingRenderPass destroyed");

        Ok(())
    }
}
