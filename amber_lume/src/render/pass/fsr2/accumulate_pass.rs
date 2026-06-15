use anyhow::{bail, Result};
use ash::vk::{AccessFlags, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::fsr2::accumulate_push_constants::AccumulatePushConstants;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::binding_layout::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};
use crate::resources::store::providers::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::resources::resource_manifest::shaders;

pub struct AccumulatePass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    scene_color: VirtualImage,
    velocity: VirtualImage,
    depth: VirtualImage,
    history_a: VirtualImage,
    history_b: VirtualImage,
}

impl AccumulatePass {
    pub fn create(
        compute_pipeline_provider: &ResourceProvider<ComputePipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        scene_color: VirtualImage,
        velocity: VirtualImage,
        depth: VirtualImage,
        history_a: VirtualImage,
        history_b: VirtualImage,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::ACCUMULATE_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for Accumulate");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: pipeline_layout_registry.get(PipelineLayoutType::General),

            scene_color,
            velocity,
            depth,
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

    fn is_enabled(&self) -> bool {
        true
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
            .read_image(
                self.depth,
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
        let depth = image_scope.get_physical_image(self.depth);
        let curr = image_scope.get_physical_image(curr_handle);
        let prev = image_scope.get_physical_image(prev_handle);

        let Some(scene_color_texture) = scene.descriptors.full else {
            return Ok(());
        };
        let Some(velocity_texture) = velocity.descriptors.full else {
            return Ok(());
        };
        let Some(depth_texture) = depth.descriptors.full else {
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
            &AccumulatePushConstants {
                scene_color_texture,
                velocity_texture,
                depth_texture,
                history_prev_texture,
                history_curr_storage,
                history_valid: context.history_valid as u32,
                display_width: width,
                display_height: height,
            },
        );

        context.dispatch_2d(width, height);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("AccumulatePass destroyed");

        Ok(())
    }
}
