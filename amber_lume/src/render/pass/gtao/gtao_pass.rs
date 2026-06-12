use anyhow::{bail, Result};
use ash::vk::{
    AccessFlags, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags,
};
use std::sync::Arc;
use arc_swap::ArcSwap;
use tracing::info;

use crate::settings::settings::EngineSettings;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::gtao::gtao_push_constants::GtaoPushConstants;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::binding_layout::pipeline_layout_registry::{
    PipelineLayoutRegistry, PipelineLayoutType,
};
use crate::resources::store::providers::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::resources::resource_manifest::shaders;

pub struct GtaoPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    depth_image: VirtualImage,
    normal_image: VirtualImage,
    gtao_image: VirtualImage,
    scene_buffer: VirtualBuffer,

    settings: Arc<ArcSwap<EngineSettings>>,
}

impl GtaoPass {
    pub fn create(
        compute_pipeline_provider: &ResourceProvider<ComputePipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        depth_image: VirtualImage,
        normal_image: VirtualImage,
        gtao_image: VirtualImage,
        scene_buffer: VirtualBuffer,
        settings: Arc<ArcSwap<EngineSettings>>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::GTAO_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for Gtao");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: pipeline_layout_registry.get(PipelineLayoutType::General),

            depth_image,
            normal_image,
            gtao_image,
            scene_buffer,

            settings,
        })
    }
}

impl Pass for GtaoPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("gtao")
    }

    fn is_enabled(&self) -> bool {
        self.settings.load().render.gtao_enabled.get()
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
        context: &PassContext,
        image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        _data: Self::PassData,
    ) -> Result<()> {
        let depth_image = image_scope.get_physical_image(self.depth_image);
        let normal_image = image_scope.get_physical_image(self.normal_image);
        let gtao_image = image_scope.get_physical_image(self.gtao_image);
        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);

        let depth_descriptor_id = depth_image
            .descriptors
            .full
            .expect("Gtao depth image must have a sampled descriptor");

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

        let settings = self.settings.load();
        let radius = settings.render.gtao_radius.get();
        let power = settings.render.gtao_power.get();

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &GtaoPushConstants::create(
                scene_buffer.device_address,
                depth_descriptor_id,
                normal_descriptor_id,
                gtao_storage_id,
                width,
                height,
                radius,
                power,
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
