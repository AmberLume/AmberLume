use crate::resource::common::resource_backend::ResourceKey;
use crate::resource::utils::hasher::hasher::Hasher;

#[derive(Clone)]
pub struct ShaderConfig {
    pub name: String,
}

impl ShaderConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        hasher.hash_string(&self.name);

        hasher.finalize()
    }
}
