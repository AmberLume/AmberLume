use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::render_pass::render_pass::RenderPass;
use crate::render::render_pass::render_pass_context::RenderPassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, DependencyFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::buffer::typed::culling_views_buffer::CullingViewGpuData;
use crate::render::resources::resource_context::ResourceContext;
use crate::render::buffer::typed::entity_buffer::EntityGpuData;
use crate::render::buffer::typed::scene_buffer::{MainCameraGpuData, SceneGpuData, ShadowCascadeGpuData};
use crate::ids::ChunkIndex;
use crate::render::render_pass::culling_indirect::culling_indirect_push_constants::CullingIndirectPushConstants;
use crate::render::render_pass::render_pass_layout::RenderView;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::persistent::persistent_resources::PersistentResources;

pub struct CullingIndirectRenderPass {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    _compute_pipeline_handle: Arc<ResRef>,

    buffer_manager: Arc<BufferManager>,
}

impl CullingIndirectRenderPass {
    pub fn create(
        resource_context: &ResourceContext,
        compute_pipeline_provider: &ResourceProvider<ComputePipelineBackend>,
        persistent_resources: &PersistentResources,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: String::from("shaders/culling_indirect/culling_indirect.comp.spv"),
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

    fn push_to_culling_views(
        &self,
        render_view: &RenderView,
        culling_views: &mut Vec<CullingViewGpuData>,
    ) {
        let chunk_index = ChunkIndex { value: culling_views.len() as u32 };

        culling_views.push(
            CullingViewGpuData::create(
                render_view.projection_view,
                self.buffer_manager.indirect_buffer.chunk(chunk_index).all().device_address(),
                self.buffer_manager.draw_count_buffer.chunk(chunk_index).get().device_address(),
                self.buffer_manager.draw_data_buffer.chunk(chunk_index).all().device_address(),
            )
        );
    }
}

impl RenderPass for CullingIndirectRenderPass {
    fn is_enabled(&self) -> bool {
        true
    }

    fn begin_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        let device = &render_pass_context.device_context.device;
        let command_buffer = render_pass_context.command_recording.command_buffer;

        let mut culling_views = Vec::new();

        self.push_to_culling_views(&render_pass_context.render_views_layout.main, &mut culling_views);
        for render_view in &render_pass_context.render_views_layout.global_shadow_cascades {
            self.push_to_culling_views(&render_view, &mut culling_views);
        }

        self.buffer_manager.culling_views_buffer
            .frame(render_pass_context.frame_index)
            .all()
            .stage(&culling_views)?;

        render_pass_context.clear_buffer(&self.buffer_manager.draw_count_buffer);

        let entities_gpu_data: Vec<EntityGpuData> = render_pass_context.world_snapshot.entities.iter().map(|entity| {
            EntityGpuData::create(entity.transform_matrix, entity.model_id)
        }).collect();
        self.buffer_manager.entity_buffer.frame(render_pass_context.frame_index).all().stage(&entities_gpu_data)?;

        let main_projection_view = render_pass_context.render_views_layout.main.projection_view;
        let main_camera_gpu_data = MainCameraGpuData::new(
            main_projection_view.to_cols_array_2d(),
            render_pass_context.world_snapshot.camera_stamp.position.to_array(),
            render_pass_context.world_snapshot.camera_stamp.near,
            render_pass_context.world_snapshot.camera_stamp.far,
        );
        let main_projection_view_inverted = main_projection_view.inverse();

        let mut shadow_cascade_count: u32 = 0;
        let mut shadow_cascades = [ShadowCascadeGpuData::default(); 4];

        for render_view in &render_pass_context.render_views_layout.global_shadow_cascades {
            shadow_cascades[shadow_cascade_count as usize] = ShadowCascadeGpuData::new(
                render_view.projection_view.to_cols_array_2d(),
                (render_view.projection_view * main_projection_view_inverted).to_cols_array_2d(),
                render_pass_context.renderer_limits.shadow_map_limits.global_cascades[shadow_cascade_count as usize].end,
            );

            shadow_cascade_count += 1;
        };

        let scene_gpu_data: SceneGpuData = SceneGpuData::create(
            main_camera_gpu_data,
            render_pass_context.world_snapshot.global_shadows_direction.to_array(),
            shadow_cascade_count,
            shadow_cascades,
        );

        render_pass_context.push_using_staging(
            &self.buffer_manager.scene_buffer.frame(render_pass_context.frame_index).get(),
            scene_gpu_data,
        )?;

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                PipelineStageFlags::TRANSFER,
                PipelineStageFlags::COMPUTE_SHADER,
                DependencyFlags::empty(),
                &[],
                &[
                    self.buffer_manager.scene_buffer.frame(render_pass_context.frame_index).barrier(
                        AccessFlags::TRANSFER_WRITE,
                        AccessFlags::SHADER_READ,
                    ),
                    self.buffer_manager.culling_views_buffer.frame(render_pass_context.frame_index).barrier(
                        AccessFlags::TRANSFER_WRITE,
                        AccessFlags::SHADER_READ,
                    ),
                    self.buffer_manager.draw_count_buffer.barrier_entire(
                        AccessFlags::TRANSFER_WRITE,
                        AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
                    ),
                    self.buffer_manager.entity_buffer.frame(render_pass_context.frame_index).barrier(
                        AccessFlags::TRANSFER_WRITE,
                        AccessFlags::SHADER_READ,
                    ),
                ],
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

        render_pass_context.push_constants(
            self.pipeline_layout,
            &CullingIndirectPushConstants::create(
                self.buffer_manager.culling_views_buffer.frame(render_pass_context.frame_index).all().device_address(),
                self.buffer_manager.entity_buffer.frame(render_pass_context.frame_index).all().device_address(),
                self.buffer_manager.submesh_buffer.all().device_address(),
                self.buffer_manager.model_buffer.all().device_address(),
                self.buffer_manager.render_stats_buffer.frame(render_pass_context.frame_index).get().device_address(),
                render_pass_context.render_views_layout.count(),
                entity_count,
            ),
        );

        let workgroups = (entity_count + 255) / 256;

        unsafe { device.cmd_dispatch(command_buffer, workgroups, 1, 1) };

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                PipelineStageFlags::COMPUTE_SHADER,
                PipelineStageFlags::DRAW_INDIRECT | PipelineStageFlags::VERTEX_SHADER,
                DependencyFlags::empty(),
                &[],
                &[
                    self.buffer_manager.draw_count_buffer.barrier_entire(
                        AccessFlags::SHADER_WRITE,
                        AccessFlags::INDIRECT_COMMAND_READ,
                    ),
                    self.buffer_manager.indirect_buffer.barrier_entire(
                        AccessFlags::SHADER_WRITE,
                        AccessFlags::INDIRECT_COMMAND_READ,
                    ),
                    self.buffer_manager.draw_data_buffer.barrier_entire(
                        AccessFlags::SHADER_WRITE,
                        AccessFlags::SHADER_READ,
                    ),
                ],
                &[],
            );
        };

        Ok(())
    }

    fn end_record_commands(&self, _render_pass_context: &RenderPassContext) -> Result<()> {
        Ok(())
    }

    fn destroy(&self) -> Result<()> {
        info!("CullingRenderPass destroyed");

        Ok(())
    }
}
