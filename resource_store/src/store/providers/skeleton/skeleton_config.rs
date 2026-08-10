use gpu_data::SkeletonBoneGPU;

use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub enum SkeletonConfig {
    Alpaca {
        resource_key: String,
    },
    InBuilt {
        name: String,
        bones: Vec<SkeletonBoneGPU>,
    },
}

impl Hash for SkeletonConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Alpaca { resource_key } => {
                0.hash(state);

                resource_key.hash(state);
            }
            Self::InBuilt { name, bones } => {
                1.hash(state);

                name.hash(state);
                bones.hash(state);
            }
        }
    }
}
