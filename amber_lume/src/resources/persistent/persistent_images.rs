use std::sync::Arc;
use anyhow::Result;
use ash::vk::{Extent3D, Format, ImageTiling, ImageType, ImageUsageFlags, SampleCountFlags, SharingMode};
use crate::render::factories::image::image_description::ImageDescription;
use crate::render::factories::image::image_view_description::ImageViewDescription;
use crate::resources::descriptor_set_manager::GlobalDescriptorSetBindings;
use crate::resources::dynamic::image::image_backend::ImageBackend;
use crate::resources::dynamic::image::image_config::ImageConfig;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::sampler_registry::SamplerType;

pub struct PersistentImages {
    pub white_pixel: Arc<ResRef>,
    pub neutral_normal: Arc<ResRef>,
    pub neutral_orm: Arc<ResRef>,
}

impl PersistentImages {
    pub fn create(
        image_provider: &ResourceProvider<ImageBackend>,
        format: Format,
        samples: SampleCountFlags,
    ) -> Result<Self> {
        let pixel_extent = Extent3D {
            width: 1,
            height: 1,
            depth: 1,
        };

        let white_pixel = image_provider.acquire_sync(ImageConfig::Inbuilt {
            label: "white_pixel".to_string(),
                image_description: ImageDescription {
                image_type: ImageType::TYPE_2D,
                format,
                extent: pixel_extent,
                mip_levels: 1,
                array_layers: 1,
                samples,
                tiling: ImageTiling::OPTIMAL,
                usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
                sharing_mode: SharingMode::EXCLUSIVE,
            },
            image_view_description: ImageViewDescription::default_2d_color(),
            binding: GlobalDescriptorSetBindings::Texture,
            sampler_type: SamplerType::LinearRepeat,
            data: Some(vec![255, 255, 255, 255]),
        });

        let neutral_normal = image_provider.acquire_sync(ImageConfig::Inbuilt {
            label: "neutral_normal".to_string(),
            image_description: ImageDescription {
                image_type: ImageType::TYPE_2D,
                format: Format::R8G8B8A8_UNORM,
                extent: pixel_extent,
                mip_levels: 1,
                array_layers: 1,
                samples,
                tiling: ImageTiling::OPTIMAL,
                usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
                sharing_mode: SharingMode::EXCLUSIVE,
            },
            image_view_description: ImageViewDescription::default_2d_color(),
            binding: GlobalDescriptorSetBindings::Texture,
            sampler_type: SamplerType::LinearRepeat,
            data: Some(vec![128, 128, 255, 0]),
        });

        let neutral_occlusion_roughness_metallic = image_provider.acquire_sync(ImageConfig::Inbuilt {
            label: "neutral_occlusion_roughness_metallic".to_string(),
            image_description: ImageDescription {
                image_type: ImageType::TYPE_2D,
                format: Format::R8G8B8A8_UNORM,
                extent: pixel_extent,
                mip_levels: 1,
                array_layers: 1,
                samples,
                tiling: ImageTiling::OPTIMAL,
                usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
                sharing_mode: SharingMode::EXCLUSIVE,
            },
            image_view_description: ImageViewDescription::default_2d_color(),
            binding: GlobalDescriptorSetBindings::Texture,
            sampler_type: SamplerType::LinearRepeat,
            data: Some(vec![255, 128, 0, 0]),
        });

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
