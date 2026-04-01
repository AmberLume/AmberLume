use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub struct ComputePipelineConfig {
    pub shader_name: String,
    pub fn_name: String,
}

impl Hash for ComputePipelineConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self { 
            shader_name,
            fn_name,
        } = self;
        
        shader_name.hash(state);
        fn_name.hash(state);
    }
}
