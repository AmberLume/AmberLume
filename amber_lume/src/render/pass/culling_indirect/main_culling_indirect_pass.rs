use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, BufferMemoryBarrier, DependencyFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, QUEUE_FAMILY_IGNORED};
use std::sync::Arc;
use tracing::info;
use crate::render::frame_data::culling_view_gpu::CullingViewGPU;
use crate::render::resources::resource_context::ResourceContext;
use crate::render::frame_data::entity_gpu::EntityGPU;
use crate::render::frame_data::scene_gpu::{MainCameraGPU, SceneGPU};
use crate::limits::ResourceLimits;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::culling_indirect::culling_indirect_push_constants::CullingIndirectPushConstants;
use crate::render::pass::culling_indirect::render_view_culling_indirect_statistics::{CullingIndirectRenderViewStatisticsGPU, MAIN_CULLING_META_NAME};
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_layout::RenderViewsLayout;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::profiler::frame_profiler::FrameProfiler;
use crate::render::statistics::meta::meta_statistics::MetaStatistics;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::resources::binding_layout::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};
use crate::resources::store::providers::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;

pub struct MainCullingIndirectPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    buffer_manager: Arc<BufferManager>,

    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    culling_view_buffer: VirtualBuffer,

    meta_statistics: Arc<MetaStatistics<CullingIndirectRenderViewStatisticsGPU>>,
}

impl MainCullingIndirectPass {
    pub fn create(
        resource_context: &ResourceContext,
        limits: &ResourceLimits,
        frame_count: u32,
        resource_factories: &ResourceFactories,
        compute_pipeline_provider: &ResourceProvider<ComputePipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        culling_view_buffer: VirtualBuffer,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: String::from("shaders/culling_indirect/culling_indirect.comp.spv"),
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline");
        };

        let meta_statistics = Arc::new(MetaStatistics::new(
            "culling_indirect",
            &resource_factories.buffer_factory,
            limits.max_render_views,
            frame_count,
        )?);

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: pipeline_layout_registry.get(PipelineLayoutType::General),

            buffer_manager: resource_context.buffer_manager.clone(),

            scene_buffer,
            entity_buffer,
            culling_view_buffer,

            meta_statistics,
        })
    }
}

pub struct MainCullingIndirectPassData {
    entity_count: usize,
}

impl Pass for MainCullingIndirectPass {
    type PassData = MainCullingIndirectPassData;

    fn name(&self) -> String {
        String::from("main_culling_indirect")
    }
    
    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(
        &self,
        context: &FrameDataContext,
        resource_registry: &mut ResourceRegistry,
        allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let entities_gpu: Vec<EntityGPU> = context.render_snapshot.entities.iter().map(|entity| {
            let is_skinned = entity.animation.is_some();

            EntityGPU::create(
                entity.transform_matrix,
                entity.mesh_id,
                is_skinned,
                entity.animation.as_ref()
                    .map(|a| a.bone_transform_offset)
                    .unwrap_or(0)
            )
        }).collect();

        self.entity_buffer.stage_slice(resource_registry, allocator, &entities_gpu)?;

        let main_projection_view = &context.render_views_layout.main.view_projection;
        let main_camera_gpu = MainCameraGPU::new(
            &main_projection_view,
            context.render_snapshot.camera.position,
            context.render_snapshot.camera.near,
            context.render_snapshot.camera.far,
        );

        let cascade_count = context.render_views_layout.cascade_count;
        let scene_gpu: SceneGPU = SceneGPU::create(
            main_camera_gpu,
            context.render_snapshot.global_shadows_direction.to_array(),
            cascade_count,
        );

        self.scene_buffer.stage_slice(resource_registry, allocator, &[scene_gpu])?;

        let main_chunk = RenderViewsLayout::get_main_index();
        let shadow_chunk = RenderViewsLayout::get_shadow_index();
        let mut culling_views = Vec::with_capacity(1 + cascade_count as usize);
        culling_views.push(CullingViewGPU::create(
            main_projection_view,
            self.buffer_manager.indirect_buffer.chunk(main_chunk),
            self.buffer_manager.draw_count_buffer.chunk(main_chunk),
            self.buffer_manager.draw_data_buffer.chunk(main_chunk),
        ));
        for _ in 0..cascade_count {
            culling_views.push(CullingViewGPU::create_for_cascade(
                self.buffer_manager.indirect_buffer.chunk(shadow_chunk),
                self.buffer_manager.draw_count_buffer.chunk(shadow_chunk),
                self.buffer_manager.draw_data_buffer.chunk(shadow_chunk),
            ));
        }

        self.culling_view_buffer.stage_slice(resource_registry, allocator, &culling_views)?;

        Ok(Self::PassData {
            entity_count: entities_gpu.len(),
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .write_buffer(
                self.scene_buffer,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .write_buffer(
                self.entity_buffer,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .write_buffer(
                self.culling_view_buffer,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            );
    }

    fn record_commands(&self, context: &PassContext, resource_registry: &ResourceRegistry, data: Self::PassData) -> Result<()> {
        if data.entity_count == 0 {
            return Ok(());
        }

        let entity_buffer = resource_registry.get_physical_buffer(self.entity_buffer);
        let culling_view_buffer = resource_registry.get_physical_buffer(self.culling_view_buffer);

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.pipeline_barrier(
            PipelineStageFlags::DRAW_INDIRECT,
            PipelineStageFlags::TRANSFER,
            DependencyFlags::empty(),
            &[],
            &[
                BufferMemoryBarrier::default()
                    .buffer(self.buffer_manager.draw_count_buffer.handle())
                    .src_access_mask(AccessFlags::INDIRECT_COMMAND_READ)
                    .dst_access_mask(AccessFlags::TRANSFER_WRITE)
                    .offset(0)
                    .size(self.buffer_manager.draw_count_buffer.entire_size())
                    .src_queue_family_index(QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(QUEUE_FAMILY_IGNORED),
            ],
            &[],
        );
        let draw_count_barrier = context.clear_buffer(
            self.buffer_manager.draw_count_buffer.as_view(),
            self.buffer_manager.draw_count_buffer.entire_size(),
            AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
        );

        let meta_statistics_barrier = self.meta_statistics.reset(&context);

        context.pipeline_barrier(
            PipelineStageFlags::TRANSFER,
            PipelineStageFlags::COMPUTE_SHADER,
            DependencyFlags::empty(),
            &[],
            &[
                draw_count_barrier,
                meta_statistics_barrier,
            ],
            &[],
        );

        context.push_constants(
            self.pipeline_layout,
            &CullingIndirectPushConstants::create(
                culling_view_buffer,
                entity_buffer,
                context.resource_buffers.mesh_buffer,
                context.resource_buffers.submesh_buffer,
                self.meta_statistics.buffer_view(context.frame_index),
                0,
                1,
                data.entity_count as u32,
                false,
            ),
        );

        context.dispatch(data.entity_count as u32);
        context.pipeline_barrier(
            PipelineStageFlags::COMPUTE_SHADER | PipelineStageFlags::TRANSFER,
            PipelineStageFlags::DRAW_INDIRECT | PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::HOST,
            DependencyFlags::empty(),
            &[],
            &[
                self.buffer_manager.draw_count_buffer.as_view().barrier(
                    AccessFlags::SHADER_WRITE | AccessFlags::TRANSFER_WRITE,
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
                self.meta_statistics.host_read_barrier(context.frame_index),
            ],
            &[],
        );

        Ok(())
    }

    fn register_with_profiler(&self, profiler: &FrameProfiler) {
        profiler.register_gpu_meta(MAIN_CULLING_META_NAME, self.meta_statistics.clone());
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("MainCullingIndirectPass destroyed");

        Ok(())
    }
}
