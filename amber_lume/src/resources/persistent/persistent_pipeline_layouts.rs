use anyhow::Result;
use ash::vk::PipelineLayout;
use crate::render::vulkan::factories::pipeline_layout::pipeline_layout_factory::PipelineLayoutFactory;
use crate::resources::persistent::persistent_descriptor_set_layouts::PersistentDescriptorSetLayouts;

pub struct PersistentPipelineLayouts {
    pub global: PipelineLayout,
}

impl PersistentPipelineLayouts {
    pub fn create(
        pipeline_layout_factory: &PipelineLayoutFactory,
        persistent_descriptor_set_layouts: &PersistentDescriptorSetLayouts,
    ) -> Result<Self> {
        let global = pipeline_layout_factory.create_pipeline_layout(
            "default",
            persistent_descriptor_set_layouts.global,
        )?;

        Ok(Self {
            global,
        })
    }

    pub fn destroy(
        self,
        pipeline_layout_factory: &PipelineLayoutFactory,
    ) {
        pipeline_layout_factory.destroy_pipeline_layout(self.global);
    }
}
