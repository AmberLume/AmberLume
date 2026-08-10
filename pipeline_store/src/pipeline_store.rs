use anyhow::Result;
use ash::vk::PipelineCache;
use index_allocator::ArcUnwrapOrErr;
use gpu::BindingLayout;
use gpu::DeviceContext;
use resource_reader::ResourceReader;
use resource_residency::ResourceProvider;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use crate::compute_pipeline_backend::ComputePipelineBackend;
use crate::pipeline_backend::PipelineBackend;
use crate::pipeline_statistics::PipelineStatistics;

pub struct PipelineStore {
    pub pipeline_provider: Arc<ResourceProvider<PipelineBackend>>,
    pub compute_pipeline_provider: Arc<ResourceProvider<ComputePipelineBackend>>,
}

impl PipelineStore {
    pub const CAPACITY: u32 = 128;

    pub fn new(
        device_context: &DeviceContext,
        binding_layout: Arc<BindingLayout>,
        resource_reader: Arc<dyn ResourceReader>,
        destroy_delay: u32,
        frame_counter: Arc<AtomicU64>,
    ) -> Self {
        let pipeline_provider = ResourceProvider::from(
            PipelineBackend::new(
                device_context.device.clone(),
                device_context.debug_utils.clone(),
                PipelineCache::null(),
                resource_reader.clone(),
                binding_layout.clone(),
            ),
            Self::CAPACITY,
            destroy_delay,
            frame_counter.clone(),
        );

        let compute_pipeline_provider = ResourceProvider::from(
            ComputePipelineBackend::new(
                device_context.device.clone(),
                device_context.debug_utils.clone(),
                PipelineCache::null(),
                resource_reader.clone(),
                binding_layout.clone(),
            ),
            Self::CAPACITY,
            destroy_delay,
            frame_counter.clone(),
        );

        Self {
            pipeline_provider,
            compute_pipeline_provider,
        }
    }

    pub fn update(&self) {
        self.pipeline_provider.update();
        self.compute_pipeline_provider.update();
    }

    pub fn statistics(&self) -> PipelineStatistics {
        PipelineStatistics {
            pipeline_provider: self.pipeline_provider.statistics(),
            compute_pipeline_provider: self.compute_pipeline_provider.statistics(),
        }
    }

    pub fn destroy(self) -> Result<()> {
        self.pipeline_provider.try_unwrap()?.destroy()?;
        self.compute_pipeline_provider.try_unwrap()?.destroy()?;

        Ok(())
    }
}
