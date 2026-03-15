use anyhow::Result;
use ash::vk::{DescriptorBindingFlags, DescriptorSetLayout, DescriptorType, ShaderStageFlags};
use crate::limits::renderer_limits::RendererLimits;
use crate::render::factories::descriptor_set_layout::descriptor_set_layout_factory::{DescriptorSetLayoutBindingDescription, DescriptorSetLayoutFactory};

#[repr(u32)]
pub enum GlobalDescriptorSetBindings {
    Texture = 0,
    TextureArray = 1,
    Shadow = 2,
    ShadowArray = 3,
}

pub struct PersistentDescriptorSetLayouts {
    pub global: DescriptorSetLayout,
}

impl PersistentDescriptorSetLayouts {
    pub fn create(
        descriptor_set_layout_factory: &DescriptorSetLayoutFactory,
        renderer_limits: &RendererLimits,
    ) -> Result<Self> {
        let global = descriptor_set_layout_factory.create_descriptor_set_layout(
            "global",
            &[
                DescriptorSetLayoutBindingDescription {
                    binding: GlobalDescriptorSetBindings::Texture as u32,
                    binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND
                        | DescriptorBindingFlags::UPDATE_AFTER_BIND,
                    descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: renderer_limits.image_resource_limits.max_texture_descriptors,
                    stage_flags: ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX | ShaderStageFlags::COMPUTE,
                },
                DescriptorSetLayoutBindingDescription {
                    binding: GlobalDescriptorSetBindings::TextureArray as u32,
                    binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND
                        | DescriptorBindingFlags::UPDATE_AFTER_BIND,
                    descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: renderer_limits.image_resource_limits.max_texture_array_descriptors,
                    stage_flags: ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX | ShaderStageFlags::COMPUTE,
                },
                DescriptorSetLayoutBindingDescription {
                    binding: GlobalDescriptorSetBindings::Shadow as u32,
                    binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND
                        | DescriptorBindingFlags::UPDATE_AFTER_BIND,
                    descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: renderer_limits.image_resource_limits.max_shadow_descriptors,
                    stage_flags: ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX | ShaderStageFlags::COMPUTE,
                },
                DescriptorSetLayoutBindingDescription {
                    binding: GlobalDescriptorSetBindings::ShadowArray as u32,
                    binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND
                        | DescriptorBindingFlags::UPDATE_AFTER_BIND,
                    descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: renderer_limits.image_resource_limits.max_shadow_array_descriptors,
                    stage_flags: ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX | ShaderStageFlags::COMPUTE,
                },
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
