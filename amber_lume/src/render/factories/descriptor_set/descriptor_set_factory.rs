use std::sync::Arc;
use ash::Device;
use anyhow::Result;
use ash::vk::{DescriptorPool, DescriptorPoolCreateFlags, DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetVariableDescriptorCountAllocateInfo, DescriptorType};
use crate::render::utils::debug_utils::DebugUtils;

pub struct DescriptorSetFactory {
    device: Device,
    debug_utils: Arc<DebugUtils>,

    descriptor_pool: DescriptorPool,
}

impl DescriptorSetFactory {
    pub fn create(
        device: Device,
        debug_utils: Arc<DebugUtils>,
    ) -> Result<Self> {
        let pool_sizes = [DescriptorPoolSize {
            ty: DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 4096,
        }];

        let pool_info = DescriptorPoolCreateInfo::default()
            .max_sets(1)
            .pool_sizes(&pool_sizes)
            .flags(DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);

        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        Ok(Self {
            device,
            debug_utils,

            descriptor_pool,
        })
    }

    pub fn create_descriptor_set(
        &self,
        label: &str,
        layouts: &[DescriptorSetLayout],
        descriptor_counts: &[u32],
    ) -> Result<DescriptorSet> {
        let mut variable_count_info = DescriptorSetVariableDescriptorCountAllocateInfo::default()
            .descriptor_counts(descriptor_counts);

        let descriptor_set_allocation_info = DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.descriptor_pool)
            .set_layouts(layouts)
            .push_next(&mut variable_count_info);

        let descriptor_set = unsafe { self.device.allocate_descriptor_sets(&descriptor_set_allocation_info)?[0] };

        self.debug_utils.label(descriptor_set, &format!("descriptor_set_{}", label));

        Ok(descriptor_set)
    }

    pub fn destroy(&self) {
        unsafe { self.device.destroy_descriptor_pool(self.descriptor_pool, None) };
    }
}