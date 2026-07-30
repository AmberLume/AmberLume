use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::shadows::shadow_resolve::shadow_resolve_push_constants::ShadowResolvePushConstants;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::resources::binding_layout::pipeline_layout_registry::PipelineLayoutType;
use crate::resources::resource_manifest::shaders;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::store::providers::res_ref::ResRef;
use crate::settings::settings::EngineSettings;
use anyhow::{bail, Result};
use arc_swap::ArcSwap;
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

    settings: Arc<ArcSwap<EngineSettings>>,
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
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::SHADOW_RESOLVE_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources
            .compute_pipeline_provider
            .acquire_sync(compute_pipeline_config);
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for ShadowResolve");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources
                .pipeline_layout_registry
                .get(PipelineLayoutType::General),

            depth_image,
            normal_image,
            shadows_image,
            output_image,
            scene_buffer,
            shadow_cascades_buffer,

            settings: resources.settings.clone(),
        })
    }
}

impl Pass for ShadowResolvePass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("shadow_resolve")
    }

    fn is_enabled(&self) -> bool {
        self.settings.load().render.shadow_enabled.value
    }

    fn prepare_data(
        &self,
        _context: &FrameDataContext,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
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
        context: &PassContext,
        image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        _data: Self::PassData,
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

        let settings = self.settings.load();
        let shadow_limits = &context.limits.shadow_map_limits;

        let frame_index = if settings.render.fsr_enabled.value {
            context.frame_number
        } else {
            0
        };

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &ShadowResolvePushConstants::create(
                &context.render_views_layout.main.jittered_view_projection,
                scene_buffer.device_address,
                shadow_cascades_buffer.device_address,
                depth_descriptor_id,
                normal_descriptor_id,
                shadow_array_descriptor_id,
                output_storage_id,
                shadow_limits.pcf_sample_count,
                frame_index,
                shadow_limits.bias,
                shadow_limits.normal_bias,
                shadow_limits.pcf_world_radius,
                shadow_limits.cascade_blend_range,
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
