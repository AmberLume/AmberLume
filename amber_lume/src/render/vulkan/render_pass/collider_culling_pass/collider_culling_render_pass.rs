use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::render_pass::render_pass::RenderPass;
use crate::render::vulkan::render_pass::render_pass_context::RenderPassContext;
use crate::resources::pipeline_layout::pipeline_layout_config::{
    PipelineLayoutConfig, PushConstantRange,
};
use crate::resources::resource_hub::ResourceHub;
use anyhow::Result;
use ash::vk::{AccessFlags, BufferMemoryBarrier, DependencyFlags, MemoryBarrier, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, ShaderStageFlags, WHOLE_SIZE};
use std::sync::Arc;
use tracing::info;
use crate::render::vulkan::buffer::typed::collider_buffer::ColliderGpuData;
use crate::render::vulkan::resource_context::ResourceContext;
use crate::render::vulkan::render_pass::collider_culling_pass::collider_culling_push_constants::ColliderCullingPushConstants;
use crate::resources::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;

pub struct ColliderCullingRenderPass {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    buffer_manager: Arc<BufferManager>,
}

impl ColliderCullingRenderPass {
    pub fn create(
        resource_context: &ResourceContext,
        resource_hub: Arc<ResourceHub>,
    ) -> Result<Self> {
        let pipeline_layout_config = PipelineLayoutConfig {
            descriptor_set_layout_configs: vec![],
            push_constant_ranges: vec![PushConstantRange {
                stage: ShaderStageFlags::COMPUTE,
                offset: 0,
                size: size_of::<ColliderCullingPushConstants>() as u32,
            }],
        };

        let pipeline_layout = *resource_hub
            .get_pipeline_layout_provider()
            .get_now(&pipeline_layout_config);

        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: String::from("shaders/collider_culling.comp.spv"),
            fn_name: String::from("main"),

            pipeline_layout_config,
        };

        let pipeline = *resource_hub
            .get_compute_pipeline_provider()
            .get_now(&compute_pipeline_config);

        Ok(Self {
            pipeline,
            pipeline_layout,

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
        self.buffer_manager.collider_buffer.stage(0, &colliders_gpu_data)?;

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
            ShaderStageFlags::COMPUTE,
            &ColliderCullingPushConstants::create(
                self.buffer_manager.scene_buffer.device_address.unwrap(),
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
            .buffer(self.buffer_manager.collider_indirect_buffer.handle)
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
