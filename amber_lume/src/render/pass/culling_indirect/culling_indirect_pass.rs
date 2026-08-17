use render_graph::ReadbackScope;
use render_graph::VirtualReadback;
use render_graph::VirtualData;
use crate::render::pass::pass_resources::PassResources;
use render_graph::Pass;
use render_graph::FrameContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, DependencyFlags, DeviceAddress, DeviceSize, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
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
    culling_view_buffer: VirtualBuffer,

    mesh_buffer: DeviceAddress,
    submesh_buffer: DeviceAddress,
    material_buffer: DeviceAddress,

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
        culling_view_buffer: VirtualBuffer,
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
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for culling_indirect");
        };

        Ok(Self {
            _handle,

            label,
            view_count,
            combine_views,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            scene_buffer,
            entity_buffer,
            culling_view_buffer,

            mesh_buffer: resources.resource_buffers.mesh_buffer,
            submesh_buffer: resources.resource_buffers.submesh_buffer,
            material_buffer: resources.resource_buffers.material_buffer,

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
                self.culling_view_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.pool.draw_count,
                AccessFlags::TRANSFER_WRITE | AccessFlags::SHADER_WRITE,
                PipelineStageFlags::TRANSFER | PipelineStageFlags::COMPUTE_SHADER,
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
        let statistics = readback_scope.get_physical_readback(self.statistics);

        let draw_count = buffer_scope.get_physical_buffer(self.pool.draw_count);

        let barriers: Vec<_> = self.requests.iter().map(|request| {
            context.clear_buffer_raw(
                draw_count.buffer,
                draw_count.offset + request.bucket.count_index as DeviceSize * size_of::<u32>() as DeviceSize,
                size_of::<u32>() as DeviceSize,
                0,
                AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
            )
        }).collect();

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
                self.mesh_buffer,
                self.submesh_buffer,
                statistics,
                cull_requests_buffer,
                buffer_scope.get_physical_buffer(self.pool.indirect),
                draw_count,
                buffer_scope.get_physical_buffer(self.pool.draw_data),
                self.material_buffer,
                scene_buffer,
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
