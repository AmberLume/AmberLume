use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, DependencyFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::frame_data::culling_view_gpu::CullingViewGPU;
use crate::render::frame_data::entity_gpu::EntityGPU;
use crate::render::frame_data::scene_gpu::{MainCameraGPU, SceneGPU};
use crate::limits::ResourceLimits;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::culling_indirect::culling_indirect_push_constants::CullingIndirectPushConstants;
use crate::render::pass::culling_indirect::render_view_culling_indirect_statistics::{CullingIndirectRenderViewStatisticsGPU, MAIN_CULLING_META_NAME};
use crate::render::pass::frame_data_context::FrameDataContext;
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
use crate::resources::resource_manifest::shaders;

pub struct MainCullingIndirectPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    culling_view_buffer: VirtualBuffer,

    draw_count_main: VirtualBuffer,
    draw_count_shadow: VirtualBuffer,
    indirect_main: VirtualBuffer,
    indirect_shadow: VirtualBuffer,
    draw_data_main: VirtualBuffer,
    draw_data_shadow: VirtualBuffer,

    meta_statistics: Arc<MetaStatistics<CullingIndirectRenderViewStatisticsGPU>>,
}

impl MainCullingIndirectPass {
    pub fn create(
        limits: &ResourceLimits,
        frame_count: u32,
        resource_factories: &ResourceFactories,
        compute_pipeline_provider: &ResourceProvider<ComputePipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        culling_view_buffer: VirtualBuffer,
        draw_count_main: VirtualBuffer,
        draw_count_shadow: VirtualBuffer,
        indirect_main: VirtualBuffer,
        indirect_shadow: VirtualBuffer,
        draw_data_main: VirtualBuffer,
        draw_data_shadow: VirtualBuffer,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::CULLING_INDIRECT_COMP,
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

            scene_buffer,
            entity_buffer,
            culling_view_buffer,

            draw_count_main,
            draw_count_shadow,
            indirect_main,
            indirect_shadow,
            draw_data_main,
            draw_data_shadow,

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
        let draw_count_main = resource_registry.get_physical_buffer(self.draw_count_main);
        let draw_count_shadow = resource_registry.get_physical_buffer(self.draw_count_shadow);
        let indirect_main = resource_registry.get_physical_buffer(self.indirect_main);
        let indirect_shadow = resource_registry.get_physical_buffer(self.indirect_shadow);
        let draw_data_main = resource_registry.get_physical_buffer(self.draw_data_main);
        let draw_data_shadow = resource_registry.get_physical_buffer(self.draw_data_shadow);

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
            context.render_snapshot.global_shadows_color.to_array(),
            context.render_snapshot.global_shadows_intensity,
            context.render_snapshot.global_ambient,
            cascade_count,
        );

        self.scene_buffer.stage_slice(resource_registry, allocator, &[scene_gpu])?;

        let mut culling_views = Vec::with_capacity(1 + cascade_count as usize);
        culling_views.push(CullingViewGPU::create(
            main_projection_view,
            indirect_main,
            draw_count_main,
            draw_data_main,
        ));
        for _ in 0..cascade_count {
            culling_views.push(CullingViewGPU::create_for_cascade(
                indirect_shadow,
                draw_count_shadow,
                draw_data_shadow,
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
            )
            .write_buffer(
                self.draw_count_main,
                AccessFlags::TRANSFER_WRITE | AccessFlags::SHADER_WRITE,
                PipelineStageFlags::TRANSFER | PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.draw_count_shadow,
                AccessFlags::TRANSFER_WRITE | AccessFlags::SHADER_WRITE,
                PipelineStageFlags::TRANSFER | PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.indirect_main,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.indirect_shadow,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.draw_data_main,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.draw_data_shadow,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            );
    }

    fn record_commands(&self, context: &PassContext, resource_registry: &ResourceRegistry, data: Self::PassData) -> Result<()> {
        if data.entity_count == 0 {
            return Ok(());
        }

        let entity_buffer = resource_registry.get_physical_buffer(self.entity_buffer);
        let culling_view_buffer = resource_registry.get_physical_buffer(self.culling_view_buffer);
        let draw_count_main = resource_registry.get_physical_buffer(self.draw_count_main);
        let draw_count_shadow = resource_registry.get_physical_buffer(self.draw_count_shadow);

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        let draw_count_main_barrier = context.clear_buffer_raw(
            draw_count_main.buffer,
            draw_count_main.offset,
            draw_count_main.size,
            AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
        );
        let draw_count_shadow_barrier = context.clear_buffer_raw(
            draw_count_shadow.buffer,
            draw_count_shadow.offset,
            draw_count_shadow.size,
            AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
        );

        let meta_statistics_barrier = self.meta_statistics.reset(&context);

        context.pipeline_barrier(
            PipelineStageFlags::TRANSFER,
            PipelineStageFlags::COMPUTE_SHADER,
            DependencyFlags::empty(),
            &[],
            &[
                draw_count_main_barrier,
                draw_count_shadow_barrier,
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
