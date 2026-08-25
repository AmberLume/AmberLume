use render_graph::ReadbackScope;
use render_graph::VirtualData;
use settings::RenderSettings;
use gpu::ResourceFactories;
use crate::render::pass::temporal_denoise::temporal_denoise_push_constants::TemporalDenoisePushConstants;
use crate::render::pass::temporal_denoise::denoise_signal::DenoiseSignal;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
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

    render_settings: VirtualData<RenderSettings>,
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
        render_settings: VirtualData<RenderSettings>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::TEMPORAL_DENOISE_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config)?;
        let Some(pipeline) = resources.compute_pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire ComputePipeline for TemporalDenoise");
        };

        Ok(Self {
            _handle,

            pipeline,
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
        
            render_settings,
        })
    }
}

pub struct TemporalDenoisePassData {
    denoise_history: u32,
}

impl Pass for TemporalDenoisePass {
    type PassData = TemporalDenoisePassData;

    fn name(&self) -> String {
        match self.signal {
            DenoiseSignal::Ao => String::from("ao_temporal_denoise"),
            DenoiseSignal::Shadow { .. } => String::from("shadow_temporal_denoise"),
        }
    }

    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool {
        let render = data_scope.get(self.render_settings);
        match self.signal {
            DenoiseSignal::Ao => render.ao_enabled.value,
            DenoiseSignal::Shadow { .. } => render.shadow_enabled.value,
        }
    }

    fn prepare_data(
        &self,
        data_scope: &mut DataResourceScope,
        _buffer_scope: &mut BufferResourceScope,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let settings = data_scope.get(self.render_settings);

        Ok(TemporalDenoisePassData {
            denoise_history: settings.denoise_history.value.round().max(1.0) as u32,
        })
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
        context: &FrameContext,
        image_scope: &ImageResourceScope,
        _buffer_scope: &BufferResourceScope,
        _readback_scope: &ReadbackScope,
        data: Self::PassData,
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
                data.denoise_history,
                context.frame_number,
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
