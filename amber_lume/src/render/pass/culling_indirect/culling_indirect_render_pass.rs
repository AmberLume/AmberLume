use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::pass::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, DependencyFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::buffer::typed::culling_views_buffer::CullingViewGPU;
use crate::render::resources::resource_context::ResourceContext;
use crate::render::buffer::typed::entity_buffer::EntityGPU;
use crate::render::buffer::typed::scene_buffer::{MainCameraGPU, SceneGPU, ShadowCascadeGPU};
use crate::ids::{ChunkIndex, FrameIndex, SliceIndex};
use crate::limits::renderer_limits::RendererLimits;
use crate::render::device::device_context::DeviceContext;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::culling_indirect::culling_indirect_push_constants::CullingIndirectPushConstants;
use crate::render::pass::culling_indirect::render_view_culling_indirect_statistics::{CullingIndirectStatistics, CullingIndirectRenderViewStatisticsGPU, CullingIndirectRenderViewStatistics};
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::statistics::interval::gpu_interval_measurement::GpuIntervalMeasurement;
use crate::render::statistics::interval::interval_measurement::IntervalMeasurement;
use crate::render::statistics::meta::meta_statistics::MetaStatistics;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};

pub struct CullingIndirectPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    buffer_manager: Arc<BufferManager>,

    culling_measurement: GpuIntervalMeasurement,
    meta_statistics: MetaStatistics<CullingIndirectRenderViewStatisticsGPU>,
}

impl CullingIndirectPass {
    pub fn create(
        device_context: &DeviceContext,
        resource_context: &ResourceContext,
        renderer_limits: &RendererLimits,
        resource_factories: &ResourceFactories,
        compute_pipeline_provider: &ResourceProvider<ComputePipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: String::from("shaders/culling_indirect/culling_indirect.comp.spv"),
            fn_name: String::from("main"),
        };

        let _handle = compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline");
        };

        let culling_measurement = GpuIntervalMeasurement::new(
            &device_context,
            "culling_indirect",
            &resource_factories.query_pool_factory,
            &resource_factories.buffer_factory,
            1,
            renderer_limits.frames_in_flight,
        )?;
        let meta_statistics = MetaStatistics::new(
            "culling_indirect",
            &resource_factories.buffer_factory,
            renderer_limits.render_resource_limits.max_render_views,
            renderer_limits.frames_in_flight,
        )?;

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: pipeline_layout_registry.get(PipelineLayoutType::General),

            buffer_manager: resource_context.buffer_manager.clone(),

            culling_measurement,
            meta_statistics,
        })
    }
    
    pub fn statistics(&self, frame_index: FrameIndex) -> CullingIndirectStatistics {
        let render_views = self.meta_statistics
            .collect(frame_index).iter()
            .map(|statistics| {
                CullingIndirectRenderViewStatistics {
                    submeshes_rendered: statistics.submeshes_rendered,
                    submeshes_culled: statistics.submeshes_culled,
                }
            })
            .collect::<Vec<_>>();
        
        let dispatch_time = self.culling_measurement.collect(frame_index)[0];
        
        CullingIndirectStatistics {
            render_views,
            
            dispatch_time,
        }
    }
}

pub struct CullingIndirectRenderPassData {
    entities_gpu: Vec<EntityGPU>,

    scene_gpu: SceneGPU,

    culling_views: Vec<CullingViewGPU>,
}

impl Pass for CullingIndirectPass {
    type PassData = CullingIndirectRenderPassData;

    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(&self, context: &FrameDataContext) -> Result<Self::PassData> {
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

        Ok(Self::PassData {
            entities_gpu,

            scene_gpu,

            culling_views,
        })
    }

    fn record_commands(&self, context: &PassContext, data: Self::PassData) -> Result<()> {
        let entity_count = data.entities_gpu.len() as u32;
        if entity_count == 0 {
            return Ok(());
        }

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        let draw_count_barrier = context.clear_buffer(
            self.buffer_manager.draw_count_buffer.as_view(),
            self.buffer_manager.draw_count_buffer.entire_size(),
            AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
        );
        let culling_views_barrier = self.buffer_manager.culling_views_buffer
            .frame(context.frame_index)
            .slice_at(SliceIndex::ZERO)
            .stage(&data.culling_views, AccessFlags::SHADER_READ)?;
        let entity_buffer_barrier = self.buffer_manager.entity_buffer
            .frame(context.frame_index)
            .slice_at(SliceIndex::ZERO)
            .stage(&data.entities_gpu, AccessFlags::SHADER_READ)?;

        let scene_barrier = context.push_using_staging(&self.buffer_manager.scene_buffer.frame(context.frame_index), data.scene_gpu, AccessFlags::SHADER_READ)?;

        let meta_statistics_barrier = self.meta_statistics.reset(&context);

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
                meta_statistics_barrier,
            ],
            &[],
        );

        context.push_constants(
            self.pipeline_layout,
            &CullingIndirectPushConstants::create(
                self.buffer_manager.culling_views_buffer.frame(context.frame_index),
                self.buffer_manager.entity_buffer.frame(context.frame_index),
                context.resource_buffers.mesh_buffer,
                context.resource_buffers.submesh_buffer,
                self.meta_statistics.buffer_view(context.frame_index),
                context.render_views_layout.count(),
                entity_count,
            ),
        );

        self.culling_measurement.reset(context.command_recording.command_buffer, context.frame_index);
        self.culling_measurement.record(
            context.command_recording.command_buffer,
            context.frame_index,
            0,
            IntervalMeasurement::Start,
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
        self.culling_measurement.record(
            context.command_recording.command_buffer,
            context.frame_index,
            0,
            IntervalMeasurement::End,
        );
        self.culling_measurement.extract(context.command_recording.command_buffer, context.frame_index);

        Ok(())
    }

    fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        info!("CullingRenderPass destroyed");

        self.culling_measurement.destroy(&resource_factories.buffer_factory)?;
        self.meta_statistics.destroy(&resource_factories.buffer_factory)?;

        Ok(())
    }
}
