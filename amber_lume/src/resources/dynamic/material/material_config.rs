use std::sync::Arc;
use bytemuck::cast_slice;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_backend::ResourceKey;
use crate::resources::utils::hasher::hasher::Hasher;

#[derive(Clone)]
pub enum MaterialConfig {
    Alpaca {
        resource_key: String,
    },
    InBuilt {
        base_color_factor: [f32; 4], 
        roughness_factor: f32,
        metallic_factor: f32,

        color_image: Arc<ResRef>,
        normal_image: Arc<ResRef>,
        orm_image: Arc<ResRef>,
    }
}

impl MaterialConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        match self {
            Self::Alpaca { resource_key } => {
                hasher.hash_u32(0);
                
                hasher.hash_string(&resource_key);
            }
            Self::InBuilt { 
                base_color_factor, 
                roughness_factor, 
                metallic_factor,

                color_image: color_texture,
                normal_image: normal_texture,
                orm_image: orm_texture,
            } => {
                hasher.hash_u32(1);
                
                hasher.hash_u8_slice(cast_slice(base_color_factor));
                hasher.hash_f32(*roughness_factor);
                hasher.hash_f32(*metallic_factor);
                
                hasher.hash_u32(color_texture.id);
                hasher.hash_u32(normal_texture.id);
                hasher.hash_u32(orm_texture.id);
            }
        }
        
        hasher.finalize()
    }
}
