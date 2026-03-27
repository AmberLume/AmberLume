use crate::resources::utils::hasher::hasher::Hasher;
use std::fmt::Debug;
use crate::resources::dynamic::resource_backend::ResourceKey;

#[derive(Clone, Debug)]
pub struct ImageConfig {
    pub resource_key: String,
}

impl ImageConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        hasher.hash_string(&self.resource_key);
        
        hasher.finalize()
    }
}
