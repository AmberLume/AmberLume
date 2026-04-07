use std::hash::{Hash, Hasher};
use std::sync::Arc;
use crate::resources::store::providers::res_ref::ResRef;

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

impl Hash for MaterialConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Alpaca { resource_key } => {
                0.hash(state);

                resource_key.hash(state);
            }
            Self::InBuilt {
                base_color_factor,
                roughness_factor,
                metallic_factor,

                color_image,
                normal_image,
                orm_image,
            } => {
                1.hash(state);
                
                for v in base_color_factor {
                    v.to_bits().hash(state);
                }
                roughness_factor.to_bits().hash(state);
                metallic_factor.to_bits().hash(state);

                color_image.id.hash(state);
                normal_image.id.hash(state);
                orm_image.id.hash(state);
            }
        }
    }
}
