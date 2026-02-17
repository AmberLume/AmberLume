use anyhow::Result;
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
    ) -> Result<Self> {
        let global = descriptor_set_factory.create_descriptor_set(
            "global",
            &[
                persistent_descriptor_set_layouts.global,
            ],
            &[
                4096,
            ]
        )?;

        Ok(Self {
            global,
        })
    }
}
