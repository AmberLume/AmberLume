use crate::factories::resource_factories::ResourceFactories;
use crate::binding_layout::descriptor_set_manager::DescriptorSetManager;
use crate::binding_layout::pipeline_layout_registry::PipelineLayoutRegistry;
use anyhow::Result;
use ash::Device;
use crate::resource_limits::ResourceLimits;

pub struct BindingLayout {
    pub descriptor_set_manager: DescriptorSetManager,
    pub pipeline_layout_registry: PipelineLayoutRegistry,
}

impl BindingLayout {
    pub fn new(
        device: Device,
        limits: &ResourceLimits,
        resource_factories: &ResourceFactories,
        ray_tracing: bool,
        frames_in_flight: u32,
    ) -> Result<Self> {
        let descriptor_set_manager = DescriptorSetManager::new(
            device,
            &resource_factories.descriptor_set_layout_factory,
            &resource_factories.descriptor_set_factory,
            &resource_factories.sampler_factory,
            &limits,
            ray_tracing,
            frames_in_flight,
        )?;

        let pipeline_layout_registry = PipelineLayoutRegistry::create(
            &resource_factories.pipeline_layout_factory,
            &descriptor_set_manager,
        )?;

        Ok(Self {
            descriptor_set_manager,
            pipeline_layout_registry,
        })
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        self.descriptor_set_manager.destroy(
            &resource_factories.sampler_factory,
            &resource_factories.descriptor_set_layout_factory,
        );
        self.pipeline_layout_registry.destroy(&resource_factories.pipeline_layout_factory);

        Ok(())
    }
}
