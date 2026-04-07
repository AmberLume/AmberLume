use crate::limits::renderer_limits::RendererLimits;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::resources::binding_layout::descriptor_set_manager::DescriptorSetManager;
use crate::resources::binding_layout::pipeline_layout_registry::PipelineLayoutRegistry;
use anyhow::Result;
use ash::Device;

pub struct BindingLayout {
    pub descriptor_set_manager: DescriptorSetManager,
    pub pipeline_layout_registry: PipelineLayoutRegistry,
}

impl BindingLayout {
    pub fn new(
        device: Device,
        renderer_limits: &RendererLimits,
        resource_factories: &ResourceFactories,
    ) -> Result<Self> {
        let descriptor_set_manager = DescriptorSetManager::new(
            device,
            &resource_factories.descriptor_set_layout_factory,
            &resource_factories.descriptor_set_factory,
            &resource_factories.sampler_factory,
            &renderer_limits,
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
