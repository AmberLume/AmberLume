use anyhow::{bail, Result};
use ash::vk::{AccessFlags, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use gpu::ResourceFactories;
use crate::render::pass::fsr2::accumulate_push_constants::AccumulatePushConstants;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use gpu::PipelineLayoutType;
use pipeline_store::ComputePipelineConfig;
use resource_residency::ResRef;
use crate::resource_manifest::shaders;

pub struct AccumulatePass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    scene_color: VirtualImage,
    velocity: VirtualImage,
    history_a: VirtualImage,
    history_b: VirtualImage,

}

impl AccumulatePass {
    pub fn create(
        resources: &PassResources,
        scene_color: VirtualImage,
        velocity: VirtualImage,
        history_a: VirtualImage,
        history_b: VirtualImage,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::ACCUMULATE_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for Accumulate");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            scene_color,
            velocity,
            history_a,
            history_b,

        })
    }
}

impl Pass for AccumulatePass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("accumulate")
    }

    fn is_enabled(&self, context: &FrameDataContext) -> bool {
        context.render_settings.fsr_enabled.value
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
                self.scene_color,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_image(
                self.velocity,
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

    fn record_commands(&self, context: &PassContext, image_scope: &ImageResourceScope, _buffer_scope: &BufferResourceScope, _data: Self::PassData) -> Result<()> {
        let (curr_handle, prev_handle) = if context.history_write_index == 0 {
            (self.history_a, self.history_b)
        } else {
            (self.history_b, self.history_a)
        };

        let scene = image_scope.get_physical_image(self.scene_color);
        let velocity = image_scope.get_physical_image(self.velocity);
        let curr = image_scope.get_physical_image(curr_handle);
        let prev = image_scope.get_physical_image(prev_handle);

        let Some(scene_color_texture) = scene.descriptors.full else {
            return Ok(());
        };
        let Some(velocity_texture) = velocity.descriptors.full else {
            return Ok(());
        };
        let Some(history_prev_texture) = prev.descriptors.full else {
            return Ok(());
        };
        let Some(history_curr_storage) = curr.descriptors.storage_mips
            .as_ref()
            .and_then(|slots| slots.first().copied())
        else {
            return Ok(());
        };

        let width = curr.extent.width;
        let height = curr.extent.height;

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &AccumulatePushConstants::create(
                scene_color_texture.inner,
                velocity_texture.inner,
                history_prev_texture.inner,
                history_curr_storage.inner,
                context.history_valid as u32,
                width,
                height,
            ),
        );

        context.dispatch_2d(width, height);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("AccumulatePass destroyed");

        Ok(())
    }
}
