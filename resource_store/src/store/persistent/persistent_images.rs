use std::sync::Arc;
use anyhow::Result;
use ash::vk::{Extent3D, Format, ImageCreateFlags, ImageTiling, ImageType, ImageUsageFlags, SampleCountFlags, SharingMode};
use gpu::ImageDescription;
use gpu::ImageViewDescription;
use resource_residency::ResRef;
use resource_residency::ResourceProvider;
use crate::store::image::backend::image_backend::ImageBackend;
use crate::store::image::backend::image_config::ImageConfig;

pub struct PersistentImages {
    pub white_pixel: Arc<ResRef>,
    pub neutral_normal: Arc<ResRef>,
    pub neutral_orm: Arc<ResRef>,
}

impl PersistentImages {
    pub fn create(image_provider: &ResourceProvider<ImageBackend>) -> Result<Self> {
        let pixel_description = ImageDescription {
            image_type: ImageType::TYPE_2D,
            format: Format::R8G8B8A8_UNORM,
            extent: Extent3D {
                width: 1,
                height: 1,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            samples: SampleCountFlags::TYPE_1,
            tiling: ImageTiling::OPTIMAL,
            usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
            sharing_mode: SharingMode::EXCLUSIVE,
            flags: ImageCreateFlags::empty(),
        };

        let white_pixel = image_provider.reserve()?;
        image_provider.write(&white_pixel, ImageConfig {
            label: "white_pixel".to_string(),
            image_description: pixel_description,
            image_view_description: ImageViewDescription::default_2d_color(),
            levels: vec![vec![255, 255, 255, 255]],
        })?;

        let neutral_normal = image_provider.reserve()?;
        image_provider.write(&neutral_normal, ImageConfig {
            label: "neutral_normal".to_string(),
            image_description: pixel_description,
            image_view_description: ImageViewDescription::default_2d_color(),
            levels: vec![vec![128, 128, 255, 0]],
        })?;

        let neutral_orm = image_provider.reserve()?;
        image_provider.write(&neutral_orm, ImageConfig {
            label: "neutral_occlusion_roughness_metallic".to_string(),
            image_description: pixel_description,
            image_view_description: ImageViewDescription::default_2d_color(),
            levels: vec![vec![255, 255, 0, 0]],
        })?;

        Ok(Self {
            white_pixel,
            neutral_normal,
            neutral_orm,
        })
    }
}
