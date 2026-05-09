use anyhow::{bail, Result};
use ash::vk::{
    AccessFlags, DependencyFlags, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout,
    PipelineStageFlags,
};
use std::sync::Arc;
use tracing::info;

use crate::ids::FrameIndex;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::frame_data::sdsm_gpu::SdsmResultGPU;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::sdsm::sdsm_push_constants::SdsmPushConstants;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::render::statistics::meta::meta_statistics::MetaStatistics;
use crate::resources::binding_layout::pipeline_layout_registry::{
    PipelineLayoutRegistry, PipelineLayoutType,
};
use crate::resources::store::providers::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;

pub struct SdsmPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    depth_image: VirtualImage,

    result_buffer: MetaStatistics<SdsmResultGPU>,
}

pub struct SdsmPassData {
    camera_near: f32,
    camera_far: f32,
}

impl SdsmPass {
    pub fn create(
        frame_count: u32,
        resource_factories: &ResourceFactories,
        compute_pipeline_provider: &ResourceProvider<ComputePipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        depth_image: VirtualImage,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: String::from("shaders/sdsm/depth_reduce.comp.spv"),
            fn_name: String::from("main"),
        };

        let _handle = compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for SDSM");
        };

        let result_buffer = MetaStatistics::new(
            "sdsm_result",
            &resource_factories.buffer_factory,
            1,
            frame_count,
        )?;

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: pipeline_layout_registry.get(PipelineLayoutType::General),

            depth_image,
            result_buffer,
        })
    }

    pub fn read_z_max(&self, frame_index: FrameIndex) -> Option<f32> {
        let result = self.result_buffer.collect(frame_index);
        if result.is_empty() {
            return None;
        }
        let bits = result[0].z_max_bits;
        if bits == 0 {
            return None;
        }
        Some(f32::from_bits(bits))
    }
}

impl Pass for SdsmPass {
    type PassData = SdsmPassData;
    type Statistics = ();

    fn name(&self) -> String {
        String::from("sdsm_reduce")
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
        Ok(SdsmPassData {
            camera_near: context.render_snapshot.camera.near,
            camera_far: context.render_snapshot.camera.far,
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration.read_image(
            self.depth_image,
            ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            AccessFlags::SHADER_READ,
            PipelineStageFlags::COMPUTE_SHADER,
        );
    }

    fn record_commands(
        &self,
        context: &PassContext,
        resource_registry: &ResourceRegistry,
        data: Self::PassData,
    ) -> Result<()> {
        let depth_image = resource_registry.get_physical_image(self.depth_image);

        let depth_width = depth_image.extent.width;
        let depth_height = depth_image.extent.height;

        const TILE_DIM: u32 = 8;
        let tile_x_count = (depth_width + TILE_DIM - 1) / TILE_DIM;
        let tile_y_count = (depth_height + TILE_DIM - 1) / TILE_DIM;
        let total_tiles = tile_x_count * tile_y_count;

        let reset_barrier = self.result_buffer.reset(context);

        context.pipeline_barrier(
            PipelineStageFlags::TRANSFER,
            PipelineStageFlags::COMPUTE_SHADER,
            DependencyFlags::empty(),
            &[],
            &[reset_barrier],
            &[],
        );

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        let depth_descriptor_id = depth_image
            .descriptor_id
            .expect("SDSM requires depth image with descriptor_id (sampled in shader)");

        context.push_constants(
            self.pipeline_layout,
            &SdsmPushConstants::create(
                self.result_buffer.buffer_view(context.frame_index),
                depth_descriptor_id,
                depth_width,
                depth_height,
                data.camera_near,
                data.camera_far,
            ),
        );

        context.dispatch(total_tiles);

        Ok(())
    }

    fn statistics(&self, _frame_index: FrameIndex) -> Self::Statistics {
        ()
    }

    fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        info!("SdsmPass destroyed");
        self.result_buffer.destroy(&resource_factories.buffer_factory)?;
        Ok(())
    }
}
