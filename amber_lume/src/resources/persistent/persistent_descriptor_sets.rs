use anyhow::Result;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::vulkan::factories::descriptor_set::descriptor_set_factory::DescriptorSetFactory;
use crate::render::vulkan::factories::descriptor_set::managed_descriptor_set::ManagedDescriptorSet;
use crate::resources::persistent::persistent_descriptor_set_layouts::PersistentDescriptorSetLayouts;

pub struct PersistentDescriptorSets {
    pub global: ManagedDescriptorSet,
}

impl PersistentDescriptorSets {
    pub fn create(
        descriptor_set_factory: &DescriptorSetFactory,
        persistent_descriptor_set_layouts: &PersistentDescriptorSetLayouts,
        renderer_limits: &RendererLimits,
    ) -> Result<Self> {
        let global = descriptor_set_factory.create_descriptor_set(
            "global",
            &[
                persistent_descriptor_set_layouts.global,
            ],
            &[
                renderer_limits.image_resource_limits.max_texture_descriptors +
                    renderer_limits.image_resource_limits.max_texture_array_descriptors +
                    renderer_limits.image_resource_limits.max_shadow_descriptors +
                    renderer_limits.image_resource_limits.max_shadow_array_descriptors,
            ]
        )?;

        Ok(Self {
            global,
        })
    }
}
