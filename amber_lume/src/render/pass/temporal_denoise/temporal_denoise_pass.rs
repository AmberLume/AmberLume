use gpu::ResourceFactories;
use crate::render::pass::temporal_denoise::temporal_denoise_push_constants::TemporalDenoisePushConstants;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::temporal_denoise::denoise_signal::DenoiseSignal;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use gpu::PipelineLayoutType;
use crate::resources::resource_manifest::shaders;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::store::providers::res_ref::ResRef;
use crate::settings::render_settings::AO_TRACE_PERIODS;
use anyhow::{bail, Result};
use ash::vk::{
    AccessFlags, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags,
};
use std::sync::Arc;
use tracing::info;

const TAU_Z: f32 = 0.1;
const TAU_N: f32 = 0.9;

pub struct TemporalDenoisePass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    noisy_image: VirtualImage,
    velocity_image: VirtualImage,
    guide_a: VirtualImage,
    guide_b: VirtualImage,
    signal_a: VirtualImage,
    signal_b: VirtualImage,

    signal: DenoiseSignal,
}

impl TemporalDenoisePass {
    pub fn create(
        resources: &PassResources,
        noisy_image: VirtualImage,
        velocity_image: VirtualImage,
        guide_a: VirtualImage,
        guide_b: VirtualImage,
        signal_a: VirtualImage,
        signal_b: VirtualImage,
        signal: DenoiseSignal,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::TEMPORAL_DENOISE_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources
            .compute_pipeline_provider
            .acquire_sync(compute_pipeline_config);
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for TemporalDenoise");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources
                .pipeline_layout_registry
                .get(PipelineLayoutType::General),

            noisy_image,
            velocity_image,
            guide_a,
            guide_b,
            signal_a,
            signal_b,
            
            signal,
        })
    }
}

impl Pass for TemporalDenoisePass {
    type PassData = ();

    fn name(&self) -> String {
        match self.signal {
            DenoiseSignal::Ao { .. } => String::from("ao_temporal_denoise"),
            DenoiseSignal::Shadow { .. } => String::from("shadow_temporal_denoise"),
        }
    }

    fn is_enabled(&self, context: &FrameDataContext) -> bool {
        let render = context.render_settings;
        match self.signal {
            DenoiseSignal::Ao { .. } => render.ao_enabled.value,
            DenoiseSignal::Shadow { .. } => render.shadow_enabled.value,
        }
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
                self.noisy_image,
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
            .read_image(
                self.guide_a,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_image(
                self.guide_b,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_image(
                self.signal_a,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_image(
                self.signal_b,
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
        let (guide_curr_handle, guide_prev_handle) = if context.history_write_index == 0 {
            (self.guide_a, self.guide_b)
        } else {
            (self.guide_b, self.guide_a)
        };
        let (signal_curr_handle, signal_prev_handle) = if context.history_write_index == 0 {
            (self.signal_a, self.signal_b)
        } else {
            (self.signal_b, self.signal_a)
        };

        let noisy_image = image_scope.get_physical_image(self.noisy_image);
        let velocity_image = image_scope.get_physical_image(self.velocity_image);
        let guide_curr = image_scope.get_physical_image(guide_curr_handle);
        let guide_prev = image_scope.get_physical_image(guide_prev_handle);
        let signal_curr = image_scope.get_physical_image(signal_curr_handle);
        let signal_prev = image_scope.get_physical_image(signal_prev_handle);

        let noisy_tex = noisy_image
            .descriptors
            .full
            .expect("TemporalDenoise input must have a sampled descriptor");

        let velocity_tex = velocity_image
            .descriptors
            .full
            .expect("TemporalDenoise velocity must have a sampled descriptor");

        let guide_curr_tex = guide_curr
            .descriptors
            .full
            .expect("TemporalDenoise guide curr must have a sampled descriptor");

        let guide_prev_tex = guide_prev
            .descriptors
            .full
            .expect("TemporalDenoise guide prev must have a sampled descriptor");

        let signal_prev_tex = signal_prev
            .descriptors
            .full
            .expect("TemporalDenoise signal prev must have a sampled descriptor");

        let signal_storage = signal_curr
            .descriptors
            .storage_mips
            .as_ref()
            .and_then(|slots| slots.first().copied())
            .expect("TemporalDenoise signal must have a storage descriptor");

        let width = signal_curr.extent.width;
        let height = signal_curr.extent.height;

        let settings = context.render_settings;
        let (trace_period, variance_clamp) = match self.signal {
            DenoiseSignal::Ao { rt_mode: true } => (
                AO_TRACE_PERIODS[settings.ao_trace_period.value.min(2)],
                1u32,
            ),
            DenoiseSignal::Ao { rt_mode: false } => (1, 0u32),
            DenoiseSignal::Shadow { .. } => (1, 1u32),
        };

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &TemporalDenoisePushConstants::create(
                guide_curr_tex.inner,
                guide_prev_tex.inner,
                velocity_tex.inner,
                noisy_tex.inner,
                signal_prev_tex.inner,
                signal_storage.inner,
                width,
                height,
                context.history_valid as u32,
                settings.denoise_history.value.round().max(1.0) as u32,
                context.frame_number,
                trace_period,
                variance_clamp,
                self.signal.is_colored() as u32,
                TAU_Z,
                TAU_N,
            ),
        );

        context.dispatch_2d(width, height);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("TemporalDenoisePass destroyed");

        Ok(())
    }
}
