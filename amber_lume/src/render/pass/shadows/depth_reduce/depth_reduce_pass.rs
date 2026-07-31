use anyhow::{bail, Result};
use ash::vk::{
    AccessFlags, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags,
};
use std::sync::Arc;
use tracing::info;

use gpu::ResourceFactories;
use crate::render::frame_data::depth_reduce_result_gpu::DepthReduceResultGPU;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::shadows::depth_reduce::depth_reduce_push_constants::DepthReducePushConstants;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use gpu::PipelineLayoutType;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::resource_manifest::shaders;

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

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for DepthReduce");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
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

    fn is_enabled(&self, _context: &FrameDataContext) -> bool {
        true
    }

    fn prepare_data(
        &self,
        _context: &FrameDataContext,
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
        context: &PassContext,
        image_scope: &ImageResourceScope, buffer_scope: &BufferResourceScope,
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
