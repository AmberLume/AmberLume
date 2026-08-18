use render_graph::ReadbackScope;
use anyhow::{bail, Result};
use ash::vk::{
    AccessFlags, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags,
};
use std::sync::Arc;
use tracing::info;
use gpu::ResourceFactories;
use crate::render::frame_data::depth_reduce_result_gpu::DepthReduceResultGPU;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::shadows::depth_reduce::depth_reduce_push_constants::DepthReducePushConstants;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::ImageResourceScope;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::HeapAllocator;
use render_graph::VirtualBuffer;
use render_graph::VirtualImage;
use gpu::PipelineLayoutType;
use pipeline_store::ComputePipelineConfig;
use resource_residency::ResRef;
use crate::resource_manifest::shaders;

pub struct DepthReducePass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    depth_image: VirtualImage,
    result_buffer: VirtualBuffer,

    stride: u32,
}

impl DepthReducePass {
    pub fn create(
        resources: &PassResources,
        depth_image: VirtualImage,
        result_buffer: VirtualBuffer,
        stride: u32,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::DEPTH_REDUCE_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config)?;
        let Some(pipeline) = resources.compute_pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire ComputePipeline for DepthReduce");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            depth_image,
            result_buffer,

            stride: stride.max(1),
        })
    }
}

impl Pass for DepthReducePass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("depth_reduce")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn prepare_data(
        &self,
        _data_scope: &mut DataResourceScope,
        buffer_scope: &mut BufferResourceScope,
        allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        self.result_buffer.stage_slice(buffer_scope, allocator, &[DepthReduceResultGPU::default()])?;

        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .read_image(
                self.depth_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.result_buffer,
                AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            );
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        image_scope: &ImageResourceScope, 
        buffer_scope: &BufferResourceScope,
        _readback_scope: &ReadbackScope,
        _data: Self::PassData,
    ) -> Result<()> {
        let depth_image = image_scope.get_physical_image(self.depth_image);
        let result_buffer = buffer_scope.get_physical_buffer(self.result_buffer);

        let depth_descriptor_id = depth_image
            .descriptors
            .full
            .expect("DepthReduce depth image must have a sampled descriptor");

        let depth_width = depth_image.extent.width;
        let depth_height = depth_image.extent.height;

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &DepthReducePushConstants::create(
                result_buffer,
                depth_descriptor_id.inner,
                depth_width,
                depth_height,
                self.stride,
            ),
        );

        let strided_width = (depth_width + self.stride - 1) / self.stride;
        let strided_height = (depth_height + self.stride - 1) / self.stride;

        context.dispatch_2d(strided_width, strided_height);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("DepthReducePass destroyed");

        Ok(())
    }
}
