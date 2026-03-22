use crate::resources::dynamic::resource_backend::ResourceKey;
use crate::resources::utils::hasher::hasher::Hasher;

#[derive(Clone, Debug)]
pub struct MeshConfig {
    pub asset_key: String,
}

impl MeshConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        hasher.hash_string(&self.asset_key);

        hasher.finalize()
    }
}
