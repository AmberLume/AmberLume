use render_graph::ReadbackScope;
use render_graph::VirtualData;
use settings::RenderSettings;
use gpu::ResourceFactories;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::ao::ao_spatial::ao_spatial_push_constants::AoSpatialPushConstants;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::HeapAllocator;
use render_graph::VirtualImage;
use render_graph::VirtualBuffer;
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

pub struct AoSpatialPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    noisy_image: VirtualImage,
    guide: [VirtualImage; 2],
    ao_image: VirtualImage,
    scene_buffer: VirtualBuffer,

    render_settings: VirtualData<RenderSettings>,
}

impl AoSpatialPass {
    pub const BLUR_RADIUS: f32 = 5.0;
    pub const PLANE_SENSITIVITY: f32 = 0.02;
    pub const NORMAL_THRESHOLD: f32 = 0.8;

    pub fn create(
        resources: &PassResources,
        noisy_image: VirtualImage,
        guide: [VirtualImage; 2],
        ao_image: VirtualImage,
        scene_buffer: VirtualBuffer,
        render_settings: VirtualData<RenderSettings>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::AO_SPATIAL_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config)?;
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for AoSpatial");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            noisy_image,
            guide,
            ao_image,
            scene_buffer,

            render_settings,
        })
    }
}

pub struct AoSpatialPassData;

impl Pass for AoSpatialPass {
    type PassData = AoSpatialPassData;

    fn name(&self) -> String {
        String::from("ao_spatial")
    }

    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool {
        data_scope.get(self.render_settings).ao_enabled.value
    }

    fn prepare_data(
        &self,
        _data_scope: &mut DataResourceScope,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        Ok(AoSpatialPassData)
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.render_settings)
            .read_image(
                self.noisy_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_image(
                self.guide[0],
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_image(
                self.guide[1],
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_image(
                self.ao_image,
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
        let guide_handle = self.guide[context.history_write_index as usize];

        let noisy_image = image_scope.get_physical_image(self.noisy_image);
        let guide_image = image_scope.get_physical_image(guide_handle);
        let ao_image = image_scope.get_physical_image(self.ao_image);

        let noisy_descriptor_id = noisy_image
            .descriptors
            .full
            .expect("AoSpatial noisy image must have a sampled descriptor");

        let guide_descriptor_id = guide_image
            .descriptors
            .full
            .expect("AoSpatial guide image must have a sampled descriptor");

        let ao_storage_id = ao_image
            .descriptors
            .storage_mips
            .as_ref()
            .and_then(|mips| mips.first().copied())
            .expect("AoSpatial ao image must have a storage descriptor");

        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);

        let width = ao_image.extent.width;
        let height = ao_image.extent.height;

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);
        context.push_constants(
            self.pipeline_layout,
            &AoSpatialPushConstants::create(
                scene_buffer.device_address,
                noisy_descriptor_id.inner,
                guide_descriptor_id.inner,
                ao_storage_id.inner,
                width,
                height,
                Self::PLANE_SENSITIVITY,
                Self::NORMAL_THRESHOLD,
                Self::BLUR_RADIUS,
                context.frame_number,
            ),
        );

        context.dispatch_2d(width, height);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        Ok(())
    }
}
