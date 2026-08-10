use gpu::ResourceFactories;
use crate::render::pass::ao::guide::denoise_guide_push_constants::DenoiseGuidePushConstants;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use gpu::PipelineLayoutType;
use crate::resource_manifest::shaders;
use pipeline_store::ComputePipelineConfig;
use resource_residency::ResRef;
use anyhow::{bail, Result};
use ash::vk::{
    AccessFlags, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags,
};
use std::sync::Arc;

pub struct DenoiseGuidePass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    depth_image: VirtualImage,
    normal_image: VirtualImage,
    guide_a: VirtualImage,
    guide_b: VirtualImage,

}

impl DenoiseGuidePass {
    pub fn create(
        resources: &PassResources,
        depth_image: VirtualImage,
        normal_image: VirtualImage,
        guide_a: VirtualImage,
        guide_b: VirtualImage,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::DENOISE_GUIDE_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources
            .compute_pipeline_provider
            .acquire_sync(compute_pipeline_config);
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for DenoiseGuide");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources
                .pipeline_layout_registry
                .get(PipelineLayoutType::General),

            depth_image,
            normal_image,
            guide_a,
            guide_b,

        })
    }
}

pub struct DenoiseGuidePassData {
    camera_position: [f32; 3],
}

impl Pass for DenoiseGuidePass {
    type PassData = DenoiseGuidePassData;

    fn name(&self) -> String {
        String::from("denoise_guide")
    }

    fn is_enabled(&self, context: &FrameDataContext) -> bool {
        let render = context.render_settings;
        render.ao_enabled.value || (render.shadow_enabled.value && render.shadow_denoise.value)
    }

    fn prepare_data(
        &self,
        context: &FrameDataContext,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        Ok(DenoiseGuidePassData {
            camera_position: context.render_snapshot.camera.position.to_array(),
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .read_image(
                self.depth_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_image(
                self.normal_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_image(
                self.guide_a,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_image(
                self.guide_b,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            );
    }

    fn record_commands(
        &self,
        context: &PassContext,
        image_scope: &ImageResourceScope,
        _buffer_scope: &BufferResourceScope,
        data: Self::PassData,
    ) -> Result<()> {
        let guide_curr_handle = if context.history_write_index == 0 {
            self.guide_a
        } else {
            self.guide_b
        };

        let depth_image = image_scope.get_physical_image(self.depth_image);
        let normal_image = image_scope.get_physical_image(self.normal_image);
        let guide_curr = image_scope.get_physical_image(guide_curr_handle);

        let depth_descriptor_id = depth_image
            .descriptors
            .full
            .expect("DenoiseGuide depth image must have a sampled descriptor");

        let normal_descriptor_id = normal_image
            .descriptors
            .full
            .expect("DenoiseGuide normal image must have a sampled descriptor");

        let guide_storage_id = guide_curr
            .descriptors
            .storage_mips
            .as_ref()
            .and_then(|slots| slots.first().copied())
            .expect("DenoiseGuide guide image must have a storage descriptor");

        let width = guide_curr.extent.width;
        let height = guide_curr.extent.height;

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);
        context.push_constants(
            self.pipeline_layout,
            &DenoiseGuidePushConstants::create(
                &context.render_views_layout.main.jittered_view_projection,
                data.camera_position,
                depth_descriptor_id.inner,
                normal_descriptor_id.inner,
                guide_storage_id.inner,
                width,
                height,
            ),
        );
        context.dispatch_2d(width, height);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        Ok(())
    }
}
