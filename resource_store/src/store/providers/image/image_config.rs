use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use gpu::ImageDescription;
use gpu::ImageViewDescription;

#[derive(Clone, Debug)]
pub enum ImageConfig {
    Alpaca {
        resource_key: String,
    },
    Inbuilt {
        label: String,

        image_description: ImageDescription,
        image_view_description: ImageViewDescription,

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

                data,
            } => {
                1.hash(state);

                label.hash(state);

                image_description.hash(state);
                image_view_description.hash(state);

                data.hash(state);
            }
        }
    }
}
