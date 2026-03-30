use crate::resources::utils::hasher::hasher::Hasher;
use std::fmt::Debug;
use crate::render::factories::image::managed_image::{ImageDescription, ImageViewDescription};
use crate::resources::descriptor_set_manager::GlobalDescriptorSetBindings;
use crate::resources::dynamic::resource_backend::ResourceKey;
use crate::resources::sampler_registry::SamplerType;

#[derive(Clone, Debug)]
pub enum ImageConfig {
    Alpaca {
        resource_key: String,
    },
    Inbuilt {
        label: String,
        
        image_description: ImageDescription,
        image_view_description: ImageViewDescription,

        binding: GlobalDescriptorSetBindings,
        sampler_type: SamplerType,

        data: Vec<u8>,
    },
}

impl ImageConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        match self {
            ImageConfig::Alpaca { resource_key } => {
                hasher.hash_u32(0);

                hasher.hash_string(&resource_key);
            }
            ImageConfig::Inbuilt { 
                label,

                image_description: _image_description,
                image_view_description: _image_view_description,

                binding,
                sampler_type,

                data,
            } => {
                hasher.hash_u32(1);

                hasher.hash_string(label);
                
                hasher.hash_u32(*binding as u32);
                hasher.hash_u32(*sampler_type as u32);

                hasher.hash_u8_slice(&data);
            }
        }
        
        hasher.finalize()
    }
}
