use render_graph::ReadbackScope;
use render_graph::VirtualData;
use anyhow::{bail, Result};
use ash::vk::{
    AccessFlags, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags,
};
use settings::RenderSettings;
use std::sync::Arc;
use tracing::info;
use gpu::ResourceFactories;
use crate::render::pass::ao::gtao::gtao_push_constants::GtaoPushConstants;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
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

pub struct GtaoPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    view_z_image: VirtualImage,
    normal_image: VirtualImage,
    gtao_image: VirtualImage,
    scene_buffer: VirtualBuffer,

    render_settings: VirtualData<RenderSettings>,
}

impl GtaoPass {
    pub fn create(
        resources: &PassResources,
        view_z_image: VirtualImage,
        normal_image: VirtualImage,
        gtao_image: VirtualImage,
        scene_buffer: VirtualBuffer,
        render_settings: VirtualData<RenderSettings>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::GTAO_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources
            .compute_pipeline_provider
            .acquire_sync(compute_pipeline_config);
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for Gtao");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources
                .pipeline_layout_registry
                .get(PipelineLayoutType::General),

            view_z_image,
            normal_image,
            gtao_image,
            scene_buffer,

            render_settings,
        })
    }
}

pub struct GtaoPassData {
    radius: f32,
    power: f32,
}

impl Pass for GtaoPass {
    type PassData = GtaoPassData;

    fn name(&self) -> String {
        String::from("gtao")
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
        let render_settings = data_scope.get(self.render_settings);

        Ok(GtaoPassData {
            radius: render_settings.gtao_radius.value,
            power: render_settings.gtao_power.value,
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.render_settings)
            .read_image(
                self.view_z_image,
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
                self.gtao_image,
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
        data: Self::PassData,
    ) -> Result<()> {
        let view_z_image = image_scope.get_physical_image(self.view_z_image);
        let normal_image = image_scope.get_physical_image(self.normal_image);
        let gtao_image = image_scope.get_physical_image(self.gtao_image);
        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);

        let normal_descriptor_id = normal_image
            .descriptors
            .full
            .expect("Gtao normal image must have a sampled descriptor");

        let gtao_storage_id = gtao_image
            .descriptors
            .storage_mips
            .as_ref()
            .and_then(|slots| slots.first().copied())
            .expect("Gtao image must have a storage descriptor");

        let width = gtao_image.extent.width;
        let height = gtao_image.extent.height;

        let temporal_index = context.frame_number;

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &GtaoPushConstants::create(
                scene_buffer.device_address,
                view_z_image
                    .descriptors
                    .full
                    .expect("Gtao view z image must have a sampled descriptor")
                    .inner,
                normal_descriptor_id.inner,
                gtao_storage_id.inner,
                width,
                height,
                temporal_index,
                view_z_image.mip_views.len() as u32,
                data.radius,
                data.power,
            ),
        );

        context.dispatch_2d(width, height);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("GtaoPass destroyed");

        Ok(())
    }
}
