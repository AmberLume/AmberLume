use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use crate::render::factories::image::image_description::ImageDescription;
use crate::render::factories::image::image_view_description::ImageViewDescription;
use crate::resources::descriptor_set_manager::GlobalDescriptorSetBindings;
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

        data: Option<Vec<u8>>,
    },
}

impl Hash for ImageConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Alpaca {
                resource_key,
            } => {
                0.hash(state);

                resource_key.hash(state);
            }
            Self::Inbuilt {
                label,

                image_description,
                image_view_description,

                binding,
                sampler_type,

                data,
            } => {
                1.hash(state);

                label.hash(state);

                image_description.hash(state);
                image_view_description.hash(state);

                (*binding as u32).hash(state);
                (*sampler_type as u32).hash(state);

                data.hash(state);
            }
        }
    }
}
