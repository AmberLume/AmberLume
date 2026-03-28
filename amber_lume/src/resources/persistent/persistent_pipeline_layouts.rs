use anyhow::Result;
use ash::vk::PipelineLayout;
use crate::render::factories::pipeline_layout::pipeline_layout_factory::PipelineLayoutFactory;
use crate::resources::descriptor_set_manager::DescriptorSetManager;

pub struct PersistentPipelineLayouts {
    pub global: PipelineLayout,
}

impl PersistentPipelineLayouts {
    pub fn create(
        pipeline_layout_factory: &PipelineLayoutFactory,
        descriptor_set_manager: &DescriptorSetManager,
    ) -> Result<Self> {
        let global = pipeline_layout_factory.create_pipeline_layout(
            "default",
            *descriptor_set_manager.layout(),
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
