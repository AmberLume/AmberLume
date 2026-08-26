use render_graph::VirtualReadback;
use gpu::ResourceFactories;
use crate::render::pass::draw_sort::draw_sort_push_constants::DrawSortPushConstants;
use statistics::DrawSortStatisticsGPU;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::DrawBucket;
use crate::render::pass::draw_pool::DrawPool;
use render_graph::PrepareScopes;
use render_graph::RecordScopes;
use render_graph::DataResourceScope;
use gpu::PipelineLayoutType;
use crate::resource_manifest::shaders;
use pipeline_store::ComputePipelineConfig;
use resource_residency::ResRef;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;

pub struct DrawSortPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    pool: DrawPool,
    source_bucket: DrawBucket,
    sorted_bucket: DrawBucket,

    statistics: VirtualReadback<DrawSortStatisticsGPU>,
}

impl DrawSortPass {
    pub fn create(
        resources: &PassResources,
        sort_capacity: u32,
        pool: DrawPool,
        source_bucket: DrawBucket,
        sorted_bucket: DrawBucket,
        statistics: VirtualReadback<DrawSortStatisticsGPU>,
    ) -> Result<Self> {
        if !sort_capacity.is_power_of_two() {
            bail!("DrawSort capacity {} must be a power of two", sort_capacity);
        }

        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::DRAW_SORT_COMP,
            fn_name: String::from("main"),
            specialization_entries: vec![(0, sort_capacity)],
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config)?;
        let Some(pipeline) = resources.compute_pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire ComputePipeline for draw_sort");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            pool,
            source_bucket,
            sorted_bucket,

            statistics,
        })
    }
}

impl Pass for DrawSortPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("draw_sort")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn prepare_data(
        &self,
        scopes: &mut PrepareScopes,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        self.pool.reserve(scopes.buffer)?;

        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .read_buffer(
                self.pool.draw_count,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.pool.draw_data,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.pool.indirect,
                AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
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

        let draw_count = scopes.buffer.get_physical_buffer(self.pool.draw_count);
        let indirect = scopes.buffer.get_physical_buffer(self.pool.indirect);
        let draw_data = scopes.buffer.get_physical_buffer(self.pool.draw_data);

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &DrawSortPushConstants::create(
                indirect.range,
                draw_count.range,
                draw_data.range,
                statistics.range,
                self.source_bucket,
                self.sorted_bucket,
            ),
        );

        context.dispatch_groups(1, 1, 1);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("DrawSortPass destroyed");

        Ok(())
    }
}
