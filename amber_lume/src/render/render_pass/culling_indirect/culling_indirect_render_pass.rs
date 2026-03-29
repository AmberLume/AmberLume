use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::render_pass::render_pass::RenderPass;
use crate::render::render_pass::render_pass_context::RenderPassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, DependencyFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::buffer::typed::culling_views_buffer::CullingViewGPU;
use crate::render::resources::resource_context::ResourceContext;
use crate::render::buffer::typed::entity_buffer::EntityGPU;
use crate::render::buffer::typed::scene_buffer::{MainCameraGPU, SceneGPU, ShadowCascadeGPU};
use crate::ids::{ChunkIndex, SliceIndex};
use crate::render::render_pass::culling_indirect::culling_indirect_push_constants::CullingIndirectPushConstants;
use crate::render::render_pass::frame_data_context::FrameDataContext;
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
}

pub struct CullingIndirectRenderPassData {
    entities_gpu: Vec<EntityGPU>,

    scene_gpu: SceneGPU,

    culling_views: Vec<CullingViewGPU>,
}

impl RenderPass for CullingIndirectRenderPass {
    type RenderPassData = CullingIndirectRenderPassData;

    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(&self, context: &FrameDataContext) -> Result<Self::RenderPassData> {
        let entities_gpu: Vec<EntityGPU> = context.render_snapshot.entities.iter().map(|entity| {
            EntityGPU::create(entity.transform_matrix, entity.mesh_id)
        }).collect();

        let main_projection_view = &context.render_views_layout.main.view_projection;
        let main_camera_gpu = MainCameraGPU::new(
            &main_projection_view,
            context.render_snapshot.camera.position(),
            context.render_snapshot.camera.near,
            context.render_snapshot.camera.far,
        );

        let main_projection_view_inverted = main_projection_view.inverted();
        let shadow_cascades_vec = context.render_views_layout.global_shadow_cascades.iter()
            .enumerate()
            .map(|(i, render_view)| {
                ShadowCascadeGPU::new(
                    &render_view.view_projection,
                    &main_projection_view_inverted,
                    context.renderer_limits.shadow_map_limits.global_cascades[i].end,
                )
            })
            .collect::<Vec<_>>();

        let mut shadow_cascades = [ShadowCascadeGPU::default(); 4];
        shadow_cascades[..shadow_cascades_vec.len()].copy_from_slice(shadow_cascades_vec.as_slice());

        let scene_gpu: SceneGPU = SceneGPU::create(
            main_camera_gpu,
            context.render_snapshot.global_shadows_direction.to_array(),
            shadow_cascades_vec.len() as u32,
            shadow_cascades,
        );

        let culling_views = context.render_views_layout.iter()
            .enumerate()
            .map(|(i, render_view)| {
                let chunk_index = ChunkIndex::from(i as u32);

                CullingViewGPU::create(
                    &render_view.view_projection,
                    self.buffer_manager.indirect_buffer.chunk(chunk_index),
                    self.buffer_manager.draw_count_buffer.chunk(chunk_index),
                    self.buffer_manager.draw_data_buffer.chunk(chunk_index),
                )
            })
            .collect::<Vec<_>>();

        Ok(Self::RenderPassData {
            entities_gpu,

            scene_gpu,

            culling_views,
        })
    }

    fn record_commands(&self, context: &RenderPassContext, data: Self::RenderPassData) -> Result<()> {
        let entity_count = data.entities_gpu.len() as u32;
        if entity_count == 0 {
            return Ok(());
        }

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        let draw_count_barrier = context.clear_buffer(&self.buffer_manager.draw_count_buffer, AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE);
        let culling_views_barrier = self.buffer_manager.culling_views_buffer
            .frame(context.frame_index)
            .slice_at(SliceIndex::ZERO)
            .stage(&data.culling_views, AccessFlags::SHADER_READ)?;
        let entity_buffer_barrier = self.buffer_manager.entity_buffer
            .frame(context.frame_index)
            .slice_at(SliceIndex::ZERO)
            .stage(&data.entities_gpu, AccessFlags::SHADER_READ)?;

        let scene_barrier = context.push_using_staging(&self.buffer_manager.scene_buffer.frame(context.frame_index), data.scene_gpu, AccessFlags::SHADER_READ)?;

        context.pipeline_barrier(
            PipelineStageFlags::HOST,
            PipelineStageFlags::COMPUTE_SHADER,
            DependencyFlags::empty(),
            &[],
            &[
                entity_buffer_barrier,
                culling_views_barrier,
            ],
            &[],
        );

        context.pipeline_barrier(
            PipelineStageFlags::TRANSFER,
            PipelineStageFlags::COMPUTE_SHADER,
            DependencyFlags::empty(),
            &[],
            &[
                draw_count_barrier,
                scene_barrier,
            ],
            &[],
        );

        context.push_constants(
            self.pipeline_layout,
            &CullingIndirectPushConstants::create(
                self.buffer_manager.culling_views_buffer.frame(context.frame_index),
                self.buffer_manager.entity_buffer.frame(context.frame_index),
                self.buffer_manager.mesh_buffer.as_view(),
                self.buffer_manager.submesh_buffer.as_view(),
                self.buffer_manager.render_stats_buffer.frame(context.frame_index),
                context.render_views_layout.count(),
                entity_count,
            ),
        );

        context.dispatch(entity_count);
        context.pipeline_barrier(
            PipelineStageFlags::COMPUTE_SHADER,
            PipelineStageFlags::DRAW_INDIRECT | PipelineStageFlags::VERTEX_SHADER,
            DependencyFlags::empty(),
            &[],
            &[
                self.buffer_manager.draw_count_buffer.as_view().barrier(
                    AccessFlags::SHADER_WRITE,
                    AccessFlags::INDIRECT_COMMAND_READ,
                ),
                self.buffer_manager.indirect_buffer.as_view().barrier(
                    AccessFlags::SHADER_WRITE,
                    AccessFlags::INDIRECT_COMMAND_READ,
                ),
                self.buffer_manager.draw_data_buffer.as_view().barrier(
                    AccessFlags::SHADER_WRITE,
                    AccessFlags::SHADER_READ,
                ),
            ],
            &[],
        );

        Ok(())
    }

    fn destroy(self) -> Result<()> {
        info!("CullingRenderPass destroyed");

        Ok(())
    }
}
