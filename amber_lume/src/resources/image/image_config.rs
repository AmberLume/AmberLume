use crate::resources::common::resource_backend::ResourceKey;
use crate::resources::sampler::sampler_config::SamplerConfig;
use crate::resources::utils::hasher::hasher::Hasher;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct ImageConfig {
    pub name: String,

    pub sampler_config: SamplerConfig,
}

impl ImageConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        hasher.hash_string(&self.name);

        hasher.hash_resource_key(&self.sampler_config.hash());

        hasher.finalize()
    }
}
