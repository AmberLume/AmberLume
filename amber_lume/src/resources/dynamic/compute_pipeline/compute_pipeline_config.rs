use crate::resources::dynamic::resource_backend::ResourceKey;
use crate::resources::utils::hasher::hasher::Hasher;

#[derive(Clone, Debug)]
pub struct ComputePipelineConfig {
    pub shader_name: String,
    pub fn_name: String,
}

impl ComputePipelineConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        hasher.hash_string(&self.shader_name);
        hasher.hash_string(&self.fn_name);

        hasher.finalize()
    }
}
