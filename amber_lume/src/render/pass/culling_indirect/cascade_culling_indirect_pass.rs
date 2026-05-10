use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, DependencyFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::resources::resource_context::ResourceContext;
use crate::ids::FrameIndex;
use crate::limits::ResourceLimits;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::culling_indirect::culling_indirect_push_constants::CullingIndirectPushConstants;
use crate::render::pass::culling_indirect::render_view_culling_indirect_statistics::{CullingIndirectStatistics, CullingIndirectRenderViewStatisticsGPU, CullingIndirectRenderViewStatistics};
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::statistics::meta::meta_statistics::MetaStatistics;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::resources::binding_layout::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};
use crate::resources::store::providers::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;

pub struct CascadeCullingIndirectPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    buffer_manager: Arc<BufferManager>,

    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    culling_view_buffer: VirtualBuffer,

    meta_statistics: MetaStatistics<CullingIndirectRenderViewStatisticsGPU>,
}

impl CascadeCullingIndirectPass {
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
            bail!("Failed to acquire ComputePipeline for cascade_culling_indirect");
        };

        let meta_statistics = MetaStatistics::new(
            "cascade_culling_indirect",
            &resource_factories.buffer_factory,
            limits.max_render_views,
            frame_count,
        )?;

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

pub struct CascadeCullingIndirectPassData {
    entity_count: usize,
    cascade_count: u32,
}

impl Pass for CascadeCullingIndirectPass {
    type PassData = CascadeCullingIndirectPassData;
    type Statistics = CullingIndirectStatistics;

    fn name(&self) -> String {
        String::from("cascade_culling_indirect")
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(
        &self,
        context: &FrameDataContext,
        _resource_registry: &mut ResourceRegistry,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        Ok(Self::PassData {
            entity_count: context.render_snapshot.entities.len(),
            cascade_count: context.render_views_layout.cascade_count,
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .read_buffer(
                self.scene_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.entity_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.culling_view_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            );
    }

    fn record_commands(&self, context: &PassContext, resource_registry: &ResourceRegistry, data: Self::PassData) -> Result<()> {
        if data.entity_count == 0 || data.cascade_count == 0 {
            return Ok(());
        }

        let entity_buffer = resource_registry.get_physical_buffer(self.entity_buffer);
        let culling_view_buffer = resource_registry.get_physical_buffer(self.culling_view_buffer);

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        let meta_statistics_barrier = self.meta_statistics.reset(&context);

        context.pipeline_barrier(
            PipelineStageFlags::TRANSFER,
            PipelineStageFlags::COMPUTE_SHADER,
            DependencyFlags::empty(),
            &[],
            &[meta_statistics_barrier],
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
                1,
                data.cascade_count,
                data.entity_count as u32,
            ),
        );

        context.dispatch(data.entity_count as u32);
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

    fn statistics(&self, frame_index: FrameIndex) -> Self::Statistics {
        let render_views = self.meta_statistics
            .collect(frame_index).iter()
            .map(|statistics| {
                CullingIndirectRenderViewStatistics {
                    submeshes_rendered: statistics.submeshes_rendered,
                    submeshes_culled: statistics.submeshes_culled,
                }
            })
            .collect::<Vec<_>>();

        Self::Statistics {
            render_views,
        }
    }

    fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        info!("CascadeCullingIndirectPass destroyed");

        self.meta_statistics.destroy(&resource_factories.buffer_factory)?;

        Ok(())
    }
}
