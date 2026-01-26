use std::fmt;
use std::fmt::{Debug, Formatter};
use ash::vk::Format;
use crate::resources::common::resource_backend::ResourceKey;
use crate::resources::sampler::sampler_config::SamplerConfig;
use crate::resources::utils::hasher::hasher::Hasher;
use fmt::Result;

#[derive(Clone, Debug)]
pub struct ImageConfig {
    pub name: String,

    pub source: ImageSource,

    pub sampler_config: SamplerConfig,
}

#[derive(Clone)]
pub enum ImageSource {
    DiskKtx2,
    Virtual {
        data: Vec<u8>,

        width: u32,
        height: u32,

        format: Format,
    }
}

impl ImageConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        hasher.hash_string(&self.name);

        match &self.source {
            ImageSource::DiskKtx2 => {
                hasher.hash_u32(0);
            }
            ImageSource::Virtual { data, width, height, format } => {
                hasher.hash_u32(1);

                hasher.hash_u32(data.len() as u32);

                hasher.hash_u32(*width);
                hasher.hash_u32(*height);

                hasher.hash_i32(format.as_raw());
            }
        }

        hasher.hash_resource_key(&self.sampler_config.hash());

        hasher.finalize()
    }
}

impl Debug for ImageSource {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        match self {
            Self::DiskKtx2 => write!(formatter, "DiskKtx2"),
            Self::Virtual { data, width, height, format } => {
                formatter.debug_struct("Virtual")
                    .field("data_len", &format_args!("{} bytes", data.len()))
                    .field("width", width)
                    .field("height", height)
                    .field("format", format)
                    .finish()
            }
        }
    }
}
