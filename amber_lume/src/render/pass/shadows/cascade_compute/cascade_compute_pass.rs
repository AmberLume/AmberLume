use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, };
use std::sync::Arc;
use tracing::info;
use ash::vk::DependencyFlags;
use index_allocator::SliceIndex;
use crate::limits::ShadowMapParams;
use gpu::FrameProfiler;
use gpu::ResourceFactories;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::shadows::cascade_compute::cascade_compute_push_constants::CascadeComputePushConstants;
use crate::render::pass::shadows::cascade_compute::cascade_statistics::{CascadeStatisticsGPU, CASCADE_COMPUTE_META_NAME};
use render_graph::Pass;
use crate::render::statistics::meta::meta_statistics::MetaStatistics;
use render_graph::PassResourceDeclaration;
use render_graph::ImageResourceScope;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::HeapAllocator;
use render_graph::VirtualBuffer;
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

    meta_statistics: Arc<MetaStatistics<CascadeStatisticsGPU>>,
}

impl CascadeComputePass {
    pub fn create(
        resources: &PassResources,
        shadow_map_limits: ShadowMapParams,
        resource_factories: &ResourceFactories,
        frame_count: u32,
        scene_buffer: VirtualBuffer,
        depth_reduce_result_buffer: VirtualBuffer,
        culling_view_buffer: VirtualBuffer,
        shadow_cascades_buffer: VirtualBuffer,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::CASCADE_COMPUTE_COMP,
            fn_name: String::from("main"),
            specialization_entries: vec![(0, shadow_map_limits.cascade_count)],
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for cascade_compute");
        };

        let meta_statistics = Arc::new(MetaStatistics::new(
            "cascade_compute",
            &resource_factories.buffer_factory,
            shadow_map_limits.cascade_count,
            frame_count,
        )?);

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            shadow_map_limits,

            scene_buffer,
            depth_reduce_result_buffer,
            culling_view_buffer,
            shadow_cascades_buffer,

            meta_statistics,
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
        _data_scope: &mut DataResourceScope,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
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
        _image_scope: &ImageResourceScope, buffer_scope: &BufferResourceScope,
        _data: Self::PassData,
    ) -> Result<()> {
        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);
        let depth_reduce_result_buffer = buffer_scope.get_physical_buffer(self.depth_reduce_result_buffer);
        let culling_view_buffer = buffer_scope.get_physical_buffer(self.culling_view_buffer);
        let shadow_cascades_buffer = buffer_scope.get_physical_buffer(self.shadow_cascades_buffer);

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &CascadeComputePushConstants::create(
                scene_buffer,
                depth_reduce_result_buffer,
                culling_view_buffer,
                shadow_cascades_buffer,
                self.meta_statistics.buffer_view(context.frame_index).slice_at(SliceIndex::ZERO).device_address(),
                self.shadow_map_limits.cascade_count,
                self.shadow_map_limits.resolution,
                self.shadow_map_limits.max_distance,
                self.shadow_map_limits.split_lambda,
                self.shadow_map_limits.shadow_caster_extension,
            ),
        );

        context.dispatch(1);

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
        profiler.register_gpu_meta(CASCADE_COMPUTE_META_NAME, self.meta_statistics.clone());
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("CascadeComputePass destroyed");

        Ok(())
    }
}
