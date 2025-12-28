use crate::resources::common::resource_backend::ResourceKey;
use crate::resources::pipeline_layout::pipeline_layout_config::PipelineLayoutConfig;
use crate::resources::utils::hasher::hasher::Hasher;

#[derive(Clone, Debug)]
pub struct ComputePipelineConfig {
    pub shader_name: String,
    pub fn_name: String,
    
    pub pipeline_layout_config: PipelineLayoutConfig,
}

impl ComputePipelineConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        hasher.hash_string(&self.shader_name);
        hasher.hash_string(&self.fn_name);
        
        hasher.hash_resource_key(&self.pipeline_layout_config.hash());

        hasher.finalize()
    }
}
