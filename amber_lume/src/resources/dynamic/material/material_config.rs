use crate::resources::dynamic::resource_backend::ResourceKey;
use crate::resources::utils::hasher::hasher::Hasher;

#[derive(Clone, Debug)]
pub struct MaterialConfig {
    pub name: String,
}

impl MaterialConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        hasher.hash_string(&self.name);

        hasher.finalize()
    }
}
