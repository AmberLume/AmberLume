use gpu::SliceIndex;
use gpu::FrameProfiler;
use gpu::ResourceFactories;
use crate::render::pass::draw_sort::draw_sort_push_constants::DrawSortPushConstants;
use crate::render::pass::draw_sort::draw_sort_statistics::{DrawSortStatisticsGPU, DRAW_SORT_META_NAME};
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::pass::draw_bucket::DrawBucket;
use crate::render::pass::draw_pool::DrawPool;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::statistics::meta::meta_statistics::MetaStatistics;
use gpu::PipelineLayoutType;
use crate::resources::resource_manifest::shaders;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::store::providers::res_ref::ResRef;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, DependencyFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;

pub struct DrawSortPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    pool: DrawPool,
    source_bucket: DrawBucket,
    sorted_bucket: DrawBucket,

    meta_statistics: Arc<MetaStatistics<DrawSortStatisticsGPU>>,
}

impl DrawSortPass {
    pub fn create(
        resources: &PassResources,
        resource_factories: &ResourceFactories,
        frame_count: u32,
        sort_capacity: u32,
        pool: DrawPool,
        source_bucket: DrawBucket,
        sorted_bucket: DrawBucket,
    ) -> Result<Self> {
        if !sort_capacity.is_power_of_two() {
            bail!("DrawSort capacity {} must be a power of two", sort_capacity);
        }

        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::DRAW_SORT_COMP,
            fn_name: String::from("main"),
            specialization_entries: vec![(0, sort_capacity)],
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for draw_sort");
        };

        let meta_statistics = Arc::new(MetaStatistics::new(
            "draw_sort",
            &resource_factories.buffer_factory,
            1,
            frame_count,
        )?);

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            pool,
            source_bucket,
            sorted_bucket,

            meta_statistics,
        })
    }
}

impl Pass for DrawSortPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("draw_sort")
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(
        &self,
        _context: &FrameDataContext,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
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
        context: &PassContext,
        _image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        _data: Self::PassData,
    ) -> Result<()> {
        let draw_count = buffer_scope.get_physical_buffer(self.pool.draw_count);
        let indirect = buffer_scope.get_physical_buffer(self.pool.indirect);
        let draw_data = buffer_scope.get_physical_buffer(self.pool.draw_data);

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
            &DrawSortPushConstants::create(
                &indirect,
                &draw_count,
                &draw_data,
                self.meta_statistics
                    .buffer_view(context.frame_index)
                    .slice_at(SliceIndex::ZERO)
                    .device_address(),
                self.source_bucket,
                self.sorted_bucket,
            ),
        );

        context.dispatch_groups(1, 1, 1);

        context.pipeline_barrier(
            PipelineStageFlags::COMPUTE_SHADER,
            PipelineStageFlags::HOST,
            DependencyFlags::empty(),
            &[],
            &[self.meta_statistics.host_read_barrier(context.frame_index)],
            &[],
        );

        Ok(())
    }

    fn register_with_profiler(&self, profiler: &FrameProfiler) {
        profiler.register_gpu_meta(DRAW_SORT_META_NAME, self.meta_statistics.clone());
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("DrawSortPass destroyed");

        Ok(())
    }
}
