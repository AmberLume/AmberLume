use crate::resources::common::resource_backend::ResourceKey;
use crate::resources::descriptor_set_layout::descriptor_set_layout_config::DescriptorSetLayoutConfig;
use crate::resources::utils::hasher::hasher::Hasher;

#[derive(Clone, Debug)]
pub struct DescriptorSetConfig {
    pub descriptor_set_layout_config: DescriptorSetLayoutConfig,
}

impl DescriptorSetConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        hasher.hash_resource_key(&self.descriptor_set_layout_config.hash());

        hasher.finalize()
    }
}
