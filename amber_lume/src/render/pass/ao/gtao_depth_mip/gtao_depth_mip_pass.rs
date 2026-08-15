use render_graph::ReadbackScope;
use render_graph::VirtualData;
use settings::RenderSettings;
use gpu::ResourceFactories;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::ao::gtao_depth_mip::gtao_depth_mip_push_constants::GtaoDepthMipPushConstants;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::HeapAllocator;
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

pub struct GtaoDepthMipPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    view_z_image: VirtualImage,
    source_mip: u32,
    destination_mip: u32,

    render_settings: VirtualData<RenderSettings>,
}

impl GtaoDepthMipPass {
    pub fn create(
        resources: &PassResources,
        view_z_image: VirtualImage,
        source_mip: u32,
        destination_mip: u32,
        render_settings: VirtualData<RenderSettings>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::GTAO_DEPTH_MIP_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for GtaoDepthMip");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            view_z_image,
            source_mip,
            destination_mip,

            render_settings,
        })
    }
}

pub struct GtaoDepthMipPassData {
    radius: f32,
}

impl Pass for GtaoDepthMipPass {
    type PassData = GtaoDepthMipPassData;

    fn name(&self) -> String {
        format!("gtao_depth_mip_{}", self.destination_mip)
    }

    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool {
        data_scope.get(self.render_settings).ao_enabled.value
    }

    fn prepare_data(
        &self,
        data_scope: &mut DataResourceScope,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        Ok(GtaoDepthMipPassData {
            radius: data_scope.get(self.render_settings).gtao_radius.value,
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.render_settings)
            .read_image_mip(
                self.view_z_image,
                self.source_mip,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_image_mip(
                self.view_z_image,
                self.destination_mip,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            );
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        image_scope: &ImageResourceScope,
        _buffer_scope: &BufferResourceScope,
        _readback_scope: &ReadbackScope,
        data: Self::PassData,
    ) -> Result<()> {
        let view_z_image = image_scope.get_physical_image(self.view_z_image);

        let source_descriptor_id = view_z_image
            .descriptors
            .sampled_mips
            .as_ref()
            .and_then(|mips| mips.get(self.source_mip as usize).copied())
            .expect("GtaoDepthMip view z image must have a sampled mip descriptor");

        let view_z_storage_id = view_z_image
            .descriptors
            .storage_mips
            .as_ref()
            .and_then(|mips| mips.get(self.destination_mip as usize).copied())
            .expect("GtaoDepthMip view z image must have a storage mip descriptor");

        let width = (view_z_image.extent.width >> self.destination_mip).max(1);
        let height = (view_z_image.extent.height >> self.destination_mip).max(1);
        let source_width = (view_z_image.extent.width >> self.source_mip).max(1);
        let source_height = (view_z_image.extent.height >> self.source_mip).max(1);

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);
        context.push_constants(
            self.pipeline_layout,
            &GtaoDepthMipPushConstants::create(
                source_descriptor_id.inner,
                view_z_storage_id.inner,
                width,
                height,
                source_width,
                source_height,
                data.radius,
            ),
        );

        context.dispatch_2d(width, height);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        Ok(())
    }
}
