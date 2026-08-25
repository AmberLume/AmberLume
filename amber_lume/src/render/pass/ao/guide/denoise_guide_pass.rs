use render_graph::ReadbackScope;
use render_graph::VirtualData;
use settings::RenderSettings;
use gpu::ResourceFactories;
use crate::render::pass::ao::guide::denoise_guide_push_constants::DenoiseGuidePushConstants;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::VirtualBuffer;
use render_graph::VirtualImage;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::ImageResourceScope;
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
    scene_buffer: VirtualBuffer,

    render_settings: VirtualData<RenderSettings>,
}

impl DenoiseGuidePass {
    pub fn create(
        resources: &PassResources,
        depth_image: VirtualImage,
        normal_image: VirtualImage,
        guide_a: VirtualImage,
        guide_b: VirtualImage,
        scene_buffer: VirtualBuffer,
        render_settings: VirtualData<RenderSettings>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::DENOISE_GUIDE_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config)?;
        let Some(pipeline) = resources.compute_pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire ComputePipeline for DenoiseGuide");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            depth_image,
            normal_image,
            guide_a,
            guide_b,
            scene_buffer,

        
            render_settings,
        })
    }
}

impl Pass for DenoiseGuidePass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("denoise_guide")
    }

    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool {
        let render_settings = data_scope.get(self.render_settings);

        render_settings.ao_enabled.value || (render_settings.shadow_enabled.value && render_settings.shadow_denoise.value)
    }

    fn prepare_data(
        &self,
        _data_scope: &mut DataResourceScope,
        _buffer_scope: &mut BufferResourceScope,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.render_settings)
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
            )
            .read_buffer(
                self.scene_buffer,
                AccessFlags::SHADER_READ,
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
        let guide_curr_handle = if context.history_write_index == 0 {
            self.guide_a
        } else {
            self.guide_b
        };

        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);

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
                scene_buffer.range,
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
