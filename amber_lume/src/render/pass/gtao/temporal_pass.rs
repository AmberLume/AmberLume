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
use crate::render::pass::gtao::temporal_push_constants::GtaoTemporalPushConstants;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::binding_layout::pipeline_layout_registry::{
    PipelineLayoutRegistry, PipelineLayoutType,
};
use crate::resources::store::providers::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::resources::resource_manifest::shaders;

pub struct GtaoTemporalPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    gtao_image: VirtualImage,
    velocity_image: VirtualImage,
    history_a: VirtualImage,
    history_b: VirtualImage,

    settings: Arc<ArcSwap<EngineSettings>>,
}

impl GtaoTemporalPass {
    pub fn create(
        compute_pipeline_provider: &ResourceProvider<ComputePipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        gtao_image: VirtualImage,
        velocity_image: VirtualImage,
        history_a: VirtualImage,
        history_b: VirtualImage,
        settings: Arc<ArcSwap<EngineSettings>>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::GTAO_TEMPORAL_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for GtaoTemporal");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: pipeline_layout_registry.get(PipelineLayoutType::General),

            gtao_image,
            velocity_image,
            history_a,
            history_b,

            settings,
        })
    }
}

impl Pass for GtaoTemporalPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("gtao_temporal")
    }

    fn is_enabled(&self) -> bool {
        self.settings.load().render.gtao_enabled.value
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
                self.gtao_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_image(
                self.velocity_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_image(
                self.history_a,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_image(
                self.history_b,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            );
    }

    fn record_commands(
        &self,
        context: &PassContext,
        image_scope: &ImageResourceScope,
        _buffer_scope: &BufferResourceScope,
        _data: Self::PassData,
    ) -> Result<()> {
        let (curr_handle, prev_handle) = if context.history_write_index == 0 {
            (self.history_a, self.history_b)
        } else {
            (self.history_b, self.history_a)
        };

        let gtao_image = image_scope.get_physical_image(self.gtao_image);
        let velocity_image = image_scope.get_physical_image(self.velocity_image);
        let curr = image_scope.get_physical_image(curr_handle);
        let prev = image_scope.get_physical_image(prev_handle);

        let gtao_texture = gtao_image
            .descriptors
            .full
            .expect("Gtao temporal input must have a sampled descriptor");

        let velocity_texture = velocity_image
            .descriptors
            .full
            .expect("Gtao temporal velocity must have a sampled descriptor");

        let history_prev_texture = prev
            .descriptors
            .full
            .expect("Gtao temporal history must have a sampled descriptor");

        let history_curr_storage = curr
            .descriptors
            .storage_mips
            .as_ref()
            .and_then(|slots| slots.first().copied())
            .expect("Gtao temporal history must have a storage descriptor");

        let width = curr.extent.width;
        let height = curr.extent.height;

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &GtaoTemporalPushConstants {
                gtao_texture,
                velocity_texture,
                history_prev_texture,
                history_curr_storage,
                history_valid: context.history_valid as u32,
                width,
                height,
            },
        );

        context.dispatch_2d(width, height);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("GtaoTemporalPass destroyed");

        Ok(())
    }
}
