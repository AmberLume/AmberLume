use crate::resources::dynamic::resource_backend::ResourceKey;
use crate::resources::utils::hasher::hasher::Hasher;

#[derive(Clone, Debug)]
pub struct ModelConfig {
    pub name: String,
}

impl ModelConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        hasher.hash_string(&self.name);

        hasher.finalize()
    }
}
