use render_graph::VirtualReadback;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, DeviceSize, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::mem::size_of;
use crate::render::frame_data::culling_view_gpu::CullingViewGPU;
use crate::render::frame_data::shadow_cascades_buffer::ShadowCascadeGPU;
use std::sync::Arc;
use tracing::info;
use crate::limits::ShadowMapParams;
use gpu::ResourceFactories;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::shadows::cascade_compute::cascade_compute_push_constants::CascadeComputePushConstants;
use statistics::CascadeStatisticsGPU;
use render_graph::Pass;
use render_graph::VirtualBuffer;
use render_graph::PassResourceDeclaration;
use render_graph::PrepareScopes;
use render_graph::RecordScopes;
use render_graph::DataResourceScope;
use gpu::PipelineLayoutType;
use pipeline_store::ComputePipelineConfig;
use resource_residency::ResRef;
use crate::resource_manifest::shaders;

pub struct CascadeComputePass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    shadow_map_limits: ShadowMapParams,

    scene_buffer: VirtualBuffer,
    depth_reduce_result_buffer: VirtualBuffer,
    culling_view_buffer: VirtualBuffer,
    shadow_cascades_buffer: VirtualBuffer,

    statistics: VirtualReadback<CascadeStatisticsGPU>,
}

impl CascadeComputePass {
    pub fn create(
        resources: &PassResources,
        shadow_map_limits: ShadowMapParams,
        scene_buffer: VirtualBuffer,
        depth_reduce_result_buffer: VirtualBuffer,
        culling_view_buffer: VirtualBuffer,
        shadow_cascades_buffer: VirtualBuffer,
        statistics: VirtualReadback<CascadeStatisticsGPU>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::CASCADE_COMPUTE_COMP,
            fn_name: String::from("main"),
            specialization_entries: vec![(0, shadow_map_limits.cascade_count)],
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config)?;
        let Some(pipeline) = resources.compute_pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire ComputePipeline for cascade_compute");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            shadow_map_limits,

            scene_buffer,
            depth_reduce_result_buffer,
            culling_view_buffer,
            shadow_cascades_buffer,

            statistics,
        })
    }
}

impl Pass for CascadeComputePass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("cascade_compute")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn prepare_data(
        &self,
        scopes: &mut PrepareScopes,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let cascade_count = self.shadow_map_limits.cascade_count as DeviceSize;

        self.shadow_cascades_buffer.reserve_region(
            scopes.buffer,
            cascade_count * size_of::<ShadowCascadeGPU>() as DeviceSize,
        )?;
        self.culling_view_buffer.reserve_region(
            scopes.buffer,
            cascade_count * size_of::<CullingViewGPU>() as DeviceSize,
        )?;

        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .read_buffer(
                self.depth_reduce_result_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.scene_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.culling_view_buffer,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.shadow_cascades_buffer,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            );
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        scopes: &RecordScopes,
        _data: Self::PassData,
    ) -> Result<()> {
        let statistics = scopes.readback.get_physical_readback(self.statistics);

        let scene_buffer = scopes.buffer.get_physical_buffer(self.scene_buffer);
        let depth_reduce_result_buffer = scopes.buffer.get_physical_buffer(self.depth_reduce_result_buffer);
        let culling_view_buffer = scopes.buffer.get_physical_buffer(self.culling_view_buffer);
        let shadow_cascades_buffer = scopes.buffer.get_physical_buffer(self.shadow_cascades_buffer);

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &CascadeComputePushConstants::create(
                scene_buffer.range,
                depth_reduce_result_buffer.range,
                culling_view_buffer.range,
                shadow_cascades_buffer.range,
                statistics.range,
                self.shadow_map_limits.cascade_count,
                self.shadow_map_limits.resolution,
                self.shadow_map_limits.max_distance,
                self.shadow_map_limits.split_lambda,
                self.shadow_map_limits.shadow_caster_extension,
            ),
        );

        context.dispatch(1);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("CascadeComputePass destroyed");

        Ok(())
    }
}
