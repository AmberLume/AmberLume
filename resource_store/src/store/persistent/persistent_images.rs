use std::sync::Arc;
use anyhow::{anyhow, Result};
use ash::vk::{Extent3D, Format, ImageCreateFlags, ImageTiling, ImageType, ImageUsageFlags, SampleCountFlags, SharingMode};
use gpu::ImageDescription;
use gpu::ImageViewDescription;
use gpu::ManagedDescriptorSet;
use resource_residency::ResRef;
use resource_residency::ResourceProvider;
use crate::store::providers::image::image_backend::ImageBackend;
use crate::store::providers::image::image_config::ImageConfig;

pub struct PersistentImages {
    pub white_pixel: Arc<ResRef>,
    pub neutral_normal: Arc<ResRef>,
    pub neutral_orm: Arc<ResRef>,
}

impl PersistentImages {
    pub fn create(
        image_provider: &ResourceProvider<ImageBackend>,
        textures_descriptor_set: &ManagedDescriptorSet,
        max_texture_descriptors: u32,
    ) -> Result<Self> {
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

        let white_pixel = image_provider.acquire_sync(ImageConfig::Inbuilt {
            label: "white_pixel".to_string(),
            image_description: pixel_description,
            image_view_description: ImageViewDescription::default_2d_color(),
            data: Some(vec![255, 255, 255, 255]),
        })?;

        let white_pixel_image_view = image_provider
            .with_resource(white_pixel.id, |image| image.image_view)
            .ok_or_else(|| anyhow!("white_pixel must be available after acquire"))?;

        textures_descriptor_set.fill_with_default(
            white_pixel_image_view,
            max_texture_descriptors,
        );

        image_provider.backend.set_default_image_view(white_pixel_image_view);

        let neutral_normal = image_provider.acquire_sync(ImageConfig::Inbuilt {
            label: "neutral_normal".to_string(),
            image_description: pixel_description,
            image_view_description: ImageViewDescription::default_2d_color(),
            data: Some(vec![128, 128, 255, 0]),
        })?;

        let neutral_occlusion_roughness_metallic = image_provider.acquire_sync(ImageConfig::Inbuilt {
            label: "neutral_occlusion_roughness_metallic".to_string(),
            image_description: pixel_description,
            image_view_description: ImageViewDescription::default_2d_color(),
            data: Some(vec![255, 255, 0, 0]),
        })?;

        Ok(Self {
            white_pixel,
            neutral_normal,
            neutral_orm: neutral_occlusion_roughness_metallic,
        })
    }
    
    pub fn destroy(self) {
        drop(self.white_pixel);
        drop(self.neutral_normal);
        drop(self.neutral_orm);
    }
}
