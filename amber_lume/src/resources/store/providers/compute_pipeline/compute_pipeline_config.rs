use std::hash::{Hash, Hasher};
use crate::data::resource_handle::ShaderResource;

#[derive(Clone, Debug)]
pub struct ComputePipelineConfig {
    pub shader_name: ShaderResource,
    pub fn_name: String,
    pub specialization_entries: Vec<(u32, u32)>,
}

impl Hash for ComputePipelineConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self {
            shader_name,
            fn_name,
            specialization_entries,
        } = self;

        shader_name.hash(state);
        fn_name.hash(state);
        specialization_entries.hash(state);
    }
}
