use crate::resources::common::resource_backend::ResourceKey;
use crate::resources::utils::hasher::hasher::Hasher;

#[derive(Clone, Debug)]
pub struct MeshConfig {
    pub name: String,
}

impl MeshConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        hasher.hash_string(&self.name);

        hasher.finalize()
    }
}
