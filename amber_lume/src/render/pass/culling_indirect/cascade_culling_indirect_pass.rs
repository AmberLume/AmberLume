use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, DependencyFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::limits::ResourceLimits;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::culling_indirect::culling_indirect_push_constants::CullingIndirectPushConstants;
use crate::render::pass::culling_indirect::render_view_culling_indirect_statistics::CullingIndirectRenderViewStatisticsGPU;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::profiler::frame_profiler::FrameProfiler;
use crate::render::statistics::meta::meta_statistics::MetaStatistics;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::binding_layout::pipeline_layout_registry::PipelineLayoutType;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::resource_manifest::shaders;
use crate::render::pass::pass_resources::PassResources;

pub struct CascadeCullingIndirectPass {
    _handle: Arc<ResRef>,

    label: &'static str,
    meta_name: &'static str,
    accept_mask: u32,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    culling_view_buffer: VirtualBuffer,
    draw_count: VirtualBuffer,
    indirect: VirtualBuffer,
    draw_data: VirtualBuffer,

    meta_statistics: Arc<MetaStatistics<CullingIndirectRenderViewStatisticsGPU>>,
}

impl CascadeCullingIndirectPass {
    pub fn create(
        resources: &PassResources,
        limits: &ResourceLimits,
        frame_count: u32,
        resource_factories: &ResourceFactories,
        label: &'static str,
        meta_name: &'static str,
        accept_mask: u32,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        culling_view_buffer: VirtualBuffer,
        draw_count: VirtualBuffer,
        indirect: VirtualBuffer,
        draw_data: VirtualBuffer,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::CULLING_INDIRECT_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for cascade_culling_indirect");
        };

        let meta_statistics = Arc::new(MetaStatistics::new(
            label,
            &resource_factories.buffer_factory,
            limits.max_render_views,
            frame_count,
        )?);

        Ok(Self {
            _handle,

            label,
            meta_name,
            accept_mask,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            scene_buffer,
            entity_buffer,
            culling_view_buffer,
            draw_count,
            indirect,
            draw_data,

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

    fn name(&self) -> String {
        String::from(self.label)
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(
        &self,
        context: &FrameDataContext,
        _buffer_scope: &mut BufferResourceScope,
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
            )
            .write_buffer(
                self.draw_count,
                AccessFlags::TRANSFER_WRITE | AccessFlags::SHADER_WRITE,
                PipelineStageFlags::TRANSFER | PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.indirect,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.draw_data,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            );
    }

    fn record_commands(&self, context: &PassContext, _image_scope: &ImageResourceScope, buffer_scope: &BufferResourceScope, data: Self::PassData) -> Result<()> {
        let draw_count = buffer_scope.get_physical_buffer(self.draw_count);

        let draw_count_barrier = context.clear_buffer_raw(
            draw_count.buffer,
            draw_count.offset,
            draw_count.size,
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

        if data.entity_count == 0 || data.cascade_count == 0 {
            return Ok(());
        }

        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);
        let entity_buffer = buffer_scope.get_physical_buffer(self.entity_buffer);
        let culling_view_buffer = buffer_scope.get_physical_buffer(self.culling_view_buffer);
        let indirect = buffer_scope.get_physical_buffer(self.indirect);
        let draw_data = buffer_scope.get_physical_buffer(self.draw_data);

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &CullingIndirectPushConstants::create(
                culling_view_buffer,
                entity_buffer,
                context.resource_buffers.mesh_buffer,
                context.resource_buffers.submesh_buffer,
                self.meta_statistics.buffer_view(context.frame_index),
                indirect,
                draw_count,
                draw_data,
                context.resource_buffers.material_buffer,
                scene_buffer,
                data.cascade_count,
                data.entity_count as u32,
                true,
                self.accept_mask,
            ),
        );

        context.dispatch(data.entity_count as u32);
        context.pipeline_barrier(
            PipelineStageFlags::COMPUTE_SHADER,
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
        profiler.register_gpu_meta(self.meta_name, self.meta_statistics.clone());
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("CascadeCullingIndirectPass destroyed");

        Ok(())
    }
}
