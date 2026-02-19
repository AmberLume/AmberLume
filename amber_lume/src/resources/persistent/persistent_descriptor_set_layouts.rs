use anyhow::Result;
use ash::vk::{DescriptorBindingFlags, DescriptorSetLayout, DescriptorType, ShaderStageFlags};
use crate::render::vulkan::factories::descriptor_set_layout::descriptor_set_layout_factory::{DescriptorSetLayoutBindingDescription, DescriptorSetLayoutFactory};

pub struct PersistentDescriptorSetLayouts {
    pub global: DescriptorSetLayout,
}

impl PersistentDescriptorSetLayouts {
    pub fn create(
        descriptor_set_layout_factory: &DescriptorSetLayoutFactory,
    ) -> Result<Self> {
        let global = descriptor_set_layout_factory.create_descriptor_set_layout(
            "global",
            &[
                DescriptorSetLayoutBindingDescription {
                    binding: 0,
                    binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND
                        | DescriptorBindingFlags::UPDATE_AFTER_BIND
                        | DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT,
                    descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 4096,
                    stage_flags: ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX | ShaderStageFlags::COMPUTE,
                }
            ]
        )?;

        Ok(Self {
            global,
        })
    }

    pub fn destroy(
        self,
        descriptor_set_layout_factory: &DescriptorSetLayoutFactory,
    ) {
        descriptor_set_layout_factory.destroy_descriptor_set_layout(self.global);
    }
}
