use render_graph::ReadbackScope;
use render_graph::VirtualData;
use crate::limits::ShadowMapParams;
use gpu::ResourceFactories;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::shadows::shadow_resolve::shadow_resolve_push_constants::ShadowResolvePushConstants;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::HeapAllocator;
use render_graph::VirtualBuffer;
use render_graph::VirtualImage;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::ImageResourceScope;
use gpu::PipelineLayoutType;
use crate::resource_manifest::shaders;
use pipeline_store::ComputePipelineConfig;
use resource_residency::ResRef;
use settings::RenderSettings;
use anyhow::{bail, Result};
use ash::vk::{
    AccessFlags, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags,
};
use std::sync::Arc;
use tracing::info;

pub struct ShadowResolvePass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    depth_image: VirtualImage,
    normal_image: VirtualImage,
    shadows_image: VirtualImage,
    output_image: VirtualImage,
    scene_buffer: VirtualBuffer,
    shadow_cascades_buffer: VirtualBuffer,

    shadow_map_limits: ShadowMapParams,

    render_settings: VirtualData<RenderSettings>,
}

impl ShadowResolvePass {
    pub fn create(
        resources: &PassResources,
        depth_image: VirtualImage,
        normal_image: VirtualImage,
        shadows_image: VirtualImage,
        output_image: VirtualImage,
        scene_buffer: VirtualBuffer,
        shadow_cascades_buffer: VirtualBuffer,
        shadow_map_limits: ShadowMapParams,
        render_settings: VirtualData<RenderSettings>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::SHADOW_RESOLVE_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config)?;
        let Some(pipeline) = resources.compute_pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire ComputePipeline for ShadowResolve");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources
                .pipeline_layout_registry
                .get(PipelineLayoutType::General),

            depth_image,
            normal_image,
            shadows_image,
            output_image,
            scene_buffer,
            shadow_cascades_buffer,

            shadow_map_limits,

            render_settings,
        })
    }
}

pub struct ShadowResolvePassData {
    fsr_enabled: bool,
}

impl Pass for ShadowResolvePass {
    type PassData = ShadowResolvePassData;

    fn name(&self) -> String {
        String::from("shadow_resolve")
    }

    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool {
        let render_settings = data_scope.get(self.render_settings);

        render_settings.shadow_enabled.value
    }

    fn prepare_data(
        &self,
        data_scope: &mut DataResourceScope,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let render_settings = data_scope.get(self.render_settings);

        Ok(ShadowResolvePassData {
            fsr_enabled: render_settings.fsr_enabled.value,
        })
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
            .read_image(
                self.shadows_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_image(
                self.output_image,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.scene_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.shadow_cascades_buffer,
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
        let depth_image = image_scope.get_physical_image(self.depth_image);
        let normal_image = image_scope.get_physical_image(self.normal_image);
        let shadows_image = image_scope.get_physical_image(self.shadows_image);
        let output_image = image_scope.get_physical_image(self.output_image);
        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);
        let shadow_cascades_buffer = buffer_scope.get_physical_buffer(self.shadow_cascades_buffer);

        let depth_descriptor_id = depth_image
            .descriptors
            .full
            .expect("ShadowResolve depth image must have a sampled descriptor");

        let normal_descriptor_id = normal_image
            .descriptors
            .full
            .expect("ShadowResolve normal image must have a sampled descriptor");

        let shadow_array_descriptor_id = shadows_image
            .descriptors
            .full
            .expect("ShadowResolve shadow array must have a sampled descriptor");

        let output_storage_id = output_image
            .descriptors
            .storage_mips
            .as_ref()
            .and_then(|slots| slots.first().copied())
            .expect("ShadowResolve output image must have a storage descriptor");

        let width = output_image.extent.width;
        let height = output_image.extent.height;

        let frame_index = if data.fsr_enabled {
            context.frame_number
        } else {
            0
        };

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &ShadowResolvePushConstants::create(
                scene_buffer.range,
                shadow_cascades_buffer.range,
                depth_descriptor_id.inner,
                normal_descriptor_id.inner,
                shadow_array_descriptor_id.inner,
                output_storage_id.inner,
                self.shadow_map_limits.pcf_sample_count,
                frame_index,
                self.shadow_map_limits.bias,
                self.shadow_map_limits.normal_bias,
                self.shadow_map_limits.pcf_world_radius,
                self.shadow_map_limits.cascade_blend_range,
            ),
        );

        context.dispatch_2d(width, height);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("ShadowResolvePass destroyed");

        Ok(())
    }
}
