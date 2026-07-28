use crate::render::pass::pass_resources::PassResources;
use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, DependencyFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::frame_data::cull_request_gpu::CullRequestGPU;
use crate::render::pass::culling_indirect::cull_request::CullRequest;
use crate::render::pass::culling_indirect::culling_indirect_push_constants::CullingIndirectPushConstants;
use crate::render::pass::culling_indirect::cull_request_statistics::CullingIndirectRequestStatisticsGPU;
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

pub struct CullingIndirectPass {
    _handle: Arc<ResRef>,

    label: &'static str,
    meta_name: &'static str,
    view_count: u32,
    combine_views: bool,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    culling_view_buffer: VirtualBuffer,

    requests: Vec<CullRequest>,
    cull_requests_buffer: VirtualBuffer,

    meta_statistics: Arc<MetaStatistics<CullingIndirectRequestStatisticsGPU>>,
}

impl CullingIndirectPass {
    pub fn create(
        resources: &PassResources,
        frame_count: u32,
        resource_factories: &ResourceFactories,
        label: &'static str,
        meta_name: &'static str,
        view_count: u32,
        combine_views: bool,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        culling_view_buffer: VirtualBuffer,
        requests: Vec<CullRequest>,
        cull_requests_buffer: VirtualBuffer,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::CULLING_INDIRECT_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for culling_indirect");
        };

        let meta_statistics = Arc::new(MetaStatistics::new(
            label,
            &resource_factories.buffer_factory,
            requests.len() as u32,
            frame_count,
        )?);

        Ok(Self {
            _handle,

            label,
            meta_name,
            view_count,
            combine_views,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            scene_buffer,
            entity_buffer,
            culling_view_buffer,

            requests,
            cull_requests_buffer,

            meta_statistics,
        })
    }
}

pub struct CullingIndirectPassData {
    entity_count: usize,
}

impl Pass for CullingIndirectPass {
    type PassData = CullingIndirectPassData;

    fn name(&self) -> String {
        String::from(self.label)
    }
    
    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(
        &self,
        context: &FrameDataContext,
        buffer_scope: &mut BufferResourceScope,
        allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let requests: Vec<CullRequestGPU> = self.requests.iter().map(|request| {
            CullRequestGPU::create(
                buffer_scope.get_physical_buffer(request.indirect),
                buffer_scope.get_physical_buffer(request.draw_count),
                buffer_scope.get_physical_buffer(request.draw_data),
                request.accept_mask,
            )
        }).collect();

        self.cull_requests_buffer.stage_slice(buffer_scope, allocator, &requests)?;

        Ok(Self::PassData {
            entity_count: context.render_snapshot.entities.len(),
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .write_buffer(
                self.cull_requests_buffer,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
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

        for request in &self.requests {
            declaration
                .write_buffer(
                    request.draw_count,
                    AccessFlags::TRANSFER_WRITE | AccessFlags::SHADER_WRITE,
                    PipelineStageFlags::TRANSFER | PipelineStageFlags::COMPUTE_SHADER,
                )
                .write_buffer(
                    request.indirect,
                    AccessFlags::SHADER_WRITE,
                    PipelineStageFlags::COMPUTE_SHADER,
                )
                .write_buffer(
                    request.draw_data,
                    AccessFlags::SHADER_WRITE,
                    PipelineStageFlags::COMPUTE_SHADER,
                );
        }
    }

    fn record_commands(&self, context: &PassContext, _image_scope: &ImageResourceScope, buffer_scope: &BufferResourceScope, data: Self::PassData) -> Result<()> {
        let mut barriers: Vec<_> = self.requests.iter().map(|request| {
            let draw_count = buffer_scope.get_physical_buffer(request.draw_count);

            context.clear_buffer_raw(
                draw_count.buffer,
                draw_count.offset,
                draw_count.size,
                AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
            )
        }).collect();

        barriers.push(self.meta_statistics.reset(&context));

        context.pipeline_barrier(
            PipelineStageFlags::TRANSFER,
            PipelineStageFlags::COMPUTE_SHADER,
            DependencyFlags::empty(),
            &[],
            &barriers,
            &[],
        );

        if data.entity_count == 0 || self.view_count == 0 {
            return Ok(());
        }

        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);
        let entity_buffer = buffer_scope.get_physical_buffer(self.entity_buffer);
        let culling_view_buffer = buffer_scope.get_physical_buffer(self.culling_view_buffer);
        let cull_requests_buffer = buffer_scope.get_physical_buffer(self.cull_requests_buffer);

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &CullingIndirectPushConstants::create(
                culling_view_buffer,
                entity_buffer,
                context.resource_buffers.mesh_buffer,
                context.resource_buffers.submesh_buffer,
                self.meta_statistics.buffer_view(context.frame_index),
                cull_requests_buffer,
                context.resource_buffers.material_buffer,
                scene_buffer,
                self.view_count,
                data.entity_count as u32,
                self.combine_views,
                self.requests.len() as u32,
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
        profiler.register_gpu_meta(self.meta_name, self.meta_statistics.clone());
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("{} destroyed", self.label);

        Ok(())
    }
}
