use std::sync::Arc;
use anyhow::{anyhow, Result};
use ash::vk::{Extent3D, Format, ImageTiling, ImageType, ImageUsageFlags, SampleCountFlags, SharingMode};
use crate::render::factories::image::image_description::ImageDescription;
use crate::render::factories::image::image_view_description::ImageViewDescription;
use crate::resources::binding_layout::descriptor_set_manager::{DescriptorSetManager, GlobalDescriptorSetBindings};
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::resources::sampler_registry::SamplerType;
use crate::resources::store::providers::image::image_backend::ImageBackend;
use crate::resources::store::providers::image::image_config::ImageConfig;

pub struct PersistentImages {
    pub white_pixel: Arc<ResRef>,
    pub neutral_normal: Arc<ResRef>,
    pub neutral_orm: Arc<ResRef>,
}

impl PersistentImages {
    pub fn create(
        image_provider: &ResourceProvider<ImageBackend>,
        descriptor_set_manager: &DescriptorSetManager,
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
        };

        let white_pixel = image_provider.acquire_sync(ImageConfig::Inbuilt {
            label: "white_pixel".to_string(),
            image_description: pixel_description,
            image_view_description: ImageViewDescription::default_2d_color(),
            binding: GlobalDescriptorSetBindings::Texture,
            sampler_type: SamplerType::LinearRepeat,
            data: Some(vec![255, 255, 255, 255]),
        });

        let white_pixel_image = image_provider
            .get_resource(white_pixel.id)
            .ok_or_else(|| anyhow!("white_pixel must be available after acquire"))?;
        descriptor_set_manager.fill_binding_with_default(
            GlobalDescriptorSetBindings::Texture,
            &white_pixel_image,
            SamplerType::LinearRepeat,
            max_texture_descriptors,
        );

        let neutral_normal = image_provider.acquire_sync(ImageConfig::Inbuilt {
            label: "neutral_normal".to_string(),
            image_description: pixel_description,
            image_view_description: ImageViewDescription::default_2d_color(),
            binding: GlobalDescriptorSetBindings::Texture,
            sampler_type: SamplerType::LinearRepeat,
            data: Some(vec![128, 128, 255, 0]),
        });

        let neutral_occlusion_roughness_metallic = image_provider.acquire_sync(ImageConfig::Inbuilt {
            label: "neutral_occlusion_roughness_metallic".to_string(),
            image_description: pixel_description,
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
