use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub enum MeshConfig {
    Alpaca {
        resource_key: String,
    },
}

impl Hash for MeshConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            MeshConfig::Alpaca { 
                resource_key,
            } => {
                0.hash(state);
                
                resource_key.hash(state);
            }
        }
    }
}
