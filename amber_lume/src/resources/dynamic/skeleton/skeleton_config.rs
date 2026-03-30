use bytemuck::cast_slice;
use crate::render::buffer::typed::skeleton::skeleton_bones_buffer::SkeletonBoneGPU;
use crate::resources::dynamic::resource_backend::ResourceKey;
use crate::resources::utils::hasher::hasher::Hasher;

#[derive(Clone, Debug)]
pub enum SkeletonConfig {
    Alpaca {
        resource_key: String,
    },
    InBuilt {
        name: String,
        bones: Vec<SkeletonBoneGPU>,
    }
}

impl SkeletonConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        match self {
            Self::Alpaca { resource_key } => {
                hasher.hash_u32(0);

                hasher.hash_string(&resource_key);
            }
            Self::InBuilt { name, bones } => {
                hasher.hash_u32(1);

                hasher.hash_string(name);
                
                hasher.hash_u32(bones.len() as u32);
                for bone in bones {
                    hasher.hash_i32(bone.parent);
                    hasher.hash_u8_slice(cast_slice(&bone.inverse_bind_matrix));
                }
            }
        }

        hasher.finalize()
    }
}
