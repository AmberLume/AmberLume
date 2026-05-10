use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, };
use std::sync::Arc;
use tracing::info;
use crate::ids::FrameIndex;
use crate::limits::ShadowMapParams;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::sdsm::cascade_compute_push_constants::CascadeComputePushConstants;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::resources::binding_layout::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};
use crate::resources::store::providers::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;

pub struct CascadeComputePass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    shadow_map_limits: ShadowMapParams,

    scene_buffer: VirtualBuffer,
    sdsm_result_buffer: VirtualBuffer,
    culling_view_buffer: VirtualBuffer,
    shadow_cascades_buffer: VirtualBuffer,

    cascade_view_offset: u32,
}

impl CascadeComputePass {
    pub fn create(
        compute_pipeline_provider: &ResourceProvider<ComputePipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        shadow_map_limits: ShadowMapParams,
        scene_buffer: VirtualBuffer,
        sdsm_result_buffer: VirtualBuffer,
        culling_view_buffer: VirtualBuffer,
        shadow_cascades_buffer: VirtualBuffer,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: String::from("shaders/sdsm/cascade_compute.comp.spv"),
            fn_name: String::from("main"),
            specialization_entries: vec![(0, shadow_map_limits.cascade_count)],
        };

        let _handle = compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for cascade_compute");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: pipeline_layout_registry.get(PipelineLayoutType::General),

            shadow_map_limits,

            scene_buffer,
            sdsm_result_buffer,
            culling_view_buffer,
            shadow_cascades_buffer,

            cascade_view_offset: 1,
        })
    }
}

impl Pass for CascadeComputePass {
    type PassData = ();
    type Statistics = ();

    fn name(&self) -> String {
        String::from("cascade_compute")
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(
        &self,
        _context: &FrameDataContext,
        _resource_registry: &mut ResourceRegistry,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .read_buffer(
                self.sdsm_result_buffer,
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
        context: &PassContext,
        resource_registry: &ResourceRegistry,
        _data: Self::PassData,
    ) -> Result<()> {
        let scene_buffer = resource_registry.get_physical_buffer(self.scene_buffer);
        let sdsm_result_buffer = resource_registry.get_physical_buffer(self.sdsm_result_buffer);
        let culling_view_buffer = resource_registry.get_physical_buffer(self.culling_view_buffer);
        let shadow_cascades_buffer = resource_registry.get_physical_buffer(self.shadow_cascades_buffer);

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &CascadeComputePushConstants::create(
                scene_buffer.device_address,
                sdsm_result_buffer.device_address,
                culling_view_buffer.device_address,
                shadow_cascades_buffer.device_address,
                self.shadow_map_limits.cascade_count,
                self.cascade_view_offset,
                self.shadow_map_limits.resolution,
                self.shadow_map_limits.light_margin,
                self.shadow_map_limits.max_distance,
                self.shadow_map_limits.split_lambda,
                self.shadow_map_limits.shadow_caster_extension,
            ),
        );

        context.dispatch(1);

        Ok(())
    }

    fn statistics(&self, _frame_index: FrameIndex) -> Self::Statistics {
        ()
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("CascadeComputePass destroyed");

        Ok(())
    }
}
