use crate::resources::common::resource_backend::ResourceKey;
use crate::resources::descriptor_set_layout::descriptor_set_layout_config::DescriptorSetLayoutConfig;
use crate::resources::utils::hasher::hasher::Hasher;
use ash::vk::ShaderStageFlags;

#[derive(Clone, Debug)]
pub struct PipelineLayoutConfig {
    pub descriptor_set_layout_configs: Vec<DescriptorSetLayoutConfig>,
    pub push_constant_ranges: Vec<PushConstantRange>,
}

#[derive(Clone, Debug)]
pub struct PushConstantRange {
    pub stage: ShaderStageFlags,
    pub offset: u32,
    pub size: u32,
}

impl PipelineLayoutConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        hasher.hash_u32(self.descriptor_set_layout_configs.len() as u32);
        for config in &self.descriptor_set_layout_configs {
            hasher.hash_resource_key(&config.hash());
        }

        hasher.hash_u32(self.push_constant_ranges.len() as u32);
        for push_constant in &self.push_constant_ranges {
            hasher.hash_u32(push_constant.stage.as_raw());
            hasher.hash_u32(push_constant.offset);
            hasher.hash_u32(push_constant.size);
        }

        hasher.finalize()
    }
}
