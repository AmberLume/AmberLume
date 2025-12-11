use crate::resources::common::resource_backend::ResourceKey;
use crate::resources::utils::hasher::hasher::Hasher;
use ash::vk::{DescriptorBindingFlags, DescriptorType, ShaderStageFlags};

#[derive(Clone, Debug)]
pub struct DescriptorSetLayoutConfig {
    pub bindings: Vec<DescriptorBindingConfig>,
}

#[derive(Clone, Debug)]
pub struct DescriptorBindingConfig {
    pub binding: u32,
    pub binding_flags: DescriptorBindingFlags,

    pub stage_flags: ShaderStageFlags,

    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
}

impl DescriptorSetLayoutConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        hasher.hash_u32(self.bindings.len() as u32);
        for binding in &self.bindings {
            hasher.hash_u32(binding.binding);
            hasher.hash_u32(binding.binding_flags.as_raw());

            hasher.hash_u32(binding.stage_flags.as_raw());

            hasher.hash_i32(binding.descriptor_type.as_raw());
            hasher.hash_u32(binding.descriptor_count);
        }

        hasher.finalize()
    }
}
