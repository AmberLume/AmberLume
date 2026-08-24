use render_graph::ReadbackScope;
use render_graph::VirtualReadback;
use render_graph::VirtualData;
use crate::render::pass::pass_resources::PassResources;
use render_graph::Pass;
use render_graph::FrameContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use render_snapshot::RenderSnapshot;
use std::sync::Arc;
use tracing::info;
use gpu::ResourceFactories;
use crate::render::frame_data::cull_request_gpu::CullRequestGPU;
use crate::render::pass::culling_indirect::cull_request::CullRequest;
use crate::render::pass::draw_pool::DrawPool;
use crate::render::pass::culling_indirect::culling_indirect_push_constants::CullingIndirectPushConstants;
use statistics::CullingIndirectRequestStatisticsGPU;
use render_graph::PassResourceDeclaration;
use render_graph::ImageResourceScope;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::HeapAllocator;
use render_graph::VirtualBuffer;
use resource_residency::ResRef;
use gpu::PipelineLayoutType;
use pipeline_store::ComputePipelineConfig;
use crate::resource_manifest::shaders;

pub struct CullingIndirectPass {
    _handle: Arc<ResRef>,

    label: &'static str,
    view_count: u32,
    combine_views: bool,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    main_culling_views_buffer: VirtualBuffer,
    mesh_buffer: VirtualBuffer,
    submesh_buffer: VirtualBuffer,
    material_buffer: VirtualBuffer,

    pool: DrawPool,
    requests: Vec<CullRequest>,
    cull_requests_buffer: VirtualBuffer,

    render_snapshot: VirtualData<RenderSnapshot>,

    statistics: VirtualReadback<CullingIndirectRequestStatisticsGPU>,
}

impl CullingIndirectPass {
    pub fn create(
        resources: &PassResources,
        label: &'static str,
        view_count: u32,
        combine_views: bool,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        main_culling_views_buffer: VirtualBuffer,
        pool: DrawPool,
        requests: Vec<CullRequest>,
        cull_requests_buffer: VirtualBuffer,
        render_snapshot: VirtualData<RenderSnapshot>,
        statistics: VirtualReadback<CullingIndirectRequestStatisticsGPU>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::CULLING_INDIRECT_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config)?;
        let Some(pipeline) = resources.compute_pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire ComputePipeline for culling_indirect");
        };

        Ok(Self {
            _handle,

            label,
            view_count,
            combine_views,

            pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            scene_buffer,
            entity_buffer,
            main_culling_views_buffer,
            mesh_buffer: resources.resource_buffer_handles.mesh_buffer,
            submesh_buffer: resources.resource_buffer_handles.submesh_buffer,
            material_buffer: resources.resource_buffer_handles.material_buffer,

            pool,
            requests,
            cull_requests_buffer,

            render_snapshot,

            statistics,
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
    
    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn prepare_data(
        &self,
        data_scope: &mut DataResourceScope,
        buffer_scope: &mut BufferResourceScope,
        allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let requests: Vec<CullRequestGPU> = self.requests.iter()
            .map(|request| CullRequestGPU::create(request.accept_mask, request.bucket))
            .collect();

        self.cull_requests_buffer.stage_slice(buffer_scope, allocator, &requests)?;

        let render_snapshot = data_scope.get(self.render_snapshot);

        Ok(Self::PassData {
            entity_count: render_snapshot.entities.len(),
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.render_snapshot)
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
                self.main_culling_views_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.mesh_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.submesh_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.material_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.pool.draw_count,
                AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.pool.indirect,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.pool.draw_data,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            );
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        _image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        readback_scope: &ReadbackScope, 
        data: Self::PassData,
    ) -> Result<()> {
        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);
        let entity_buffer = buffer_scope.get_physical_buffer(self.entity_buffer);
        let main_culling_views_buffer = buffer_scope.get_physical_buffer(self.main_culling_views_buffer);
        let cull_requests_buffer = buffer_scope.get_physical_buffer(self.cull_requests_buffer);
        let draw_count = buffer_scope.get_physical_buffer(self.pool.draw_count);
        let mesh_buffer = buffer_scope.get_physical_buffer(self.mesh_buffer);
        let submesh_buffer = buffer_scope.get_physical_buffer(self.submesh_buffer);
        let indirect = buffer_scope.get_physical_buffer(self.pool.indirect);
        let draw_data = buffer_scope.get_physical_buffer(self.pool.draw_data);
        let material_buffer = buffer_scope.get_physical_buffer(self.material_buffer);
        let statistics = readback_scope.get_physical_readback(self.statistics);

        if data.entity_count == 0 || self.view_count == 0 {
            return Ok(());
        }

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &CullingIndirectPushConstants::create(
                main_culling_views_buffer.range,
                entity_buffer.range,
                mesh_buffer.range,
                submesh_buffer.range,
                statistics.range,
                cull_requests_buffer.range,
                indirect.range,
                draw_count.range,
                draw_data.range,
                material_buffer.range,
                scene_buffer.range,
                self.view_count,
                data.entity_count as u32,
                self.combine_views,
                self.requests.len() as u32,
            ),
        );

        context.dispatch(data.entity_count as u32);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("{} destroyed", self.label);

        Ok(())
    }
}
