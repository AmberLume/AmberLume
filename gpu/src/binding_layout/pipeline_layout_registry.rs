use ash::vk::PipelineLayout;
use crate::factories::pipeline_layout::pipeline_layout_factory::PipelineLayoutFactory;
use crate::binding_layout::descriptor_set_manager::DescriptorSetManager;
use anyhow::Result;

#[derive(Clone, Debug)]
pub enum PipelineLayoutType {
    General,
}

pub struct PipelineLayoutRegistry {
    general: PipelineLayout,
}

impl PipelineLayoutRegistry {
    pub fn create(
        pipeline_layout_factory: &PipelineLayoutFactory,
        descriptor_set_manager: &DescriptorSetManager,
    ) -> Result<Self> {
        let general = pipeline_layout_factory.create_pipeline_layout(
            "general",
            *descriptor_set_manager.layout(),
        )?;

        Ok(Self {
            general,
        })
    }

    pub fn get(&self, pipeline_type: PipelineLayoutType) -> PipelineLayout {
        match pipeline_type {
            PipelineLayoutType::General => self.general,
        }
    }

    pub fn destroy(self, pipeline_layout_factory: &PipelineLayoutFactory) {
        pipeline_layout_factory.destroy_pipeline_layout(self.general);
    }
}
