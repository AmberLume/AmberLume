use std::sync::Arc;
use crate::render::factories::image::managed_image::ManagedImage;
use anyhow::Result;
use ash::Instance;
use ash::vk::{Extent2D, Extent3D, Format, ImageAspectFlags, ImageTiling, ImageType, ImageUsageFlags, ImageViewType, PhysicalDevice, SampleCountFlags, SharingMode};
use crate::render::factories::image::image_description::ImageDescription;
use crate::render::factories::image::image_view_description::ImageViewDescription;
use crate::render::pass::depth::depth_format::find_depth_format;
use crate::resources::descriptor_set_manager::GlobalDescriptorSetBindings;
use crate::resources::dynamic::image::image_backend::ImageBackend;
use crate::resources::dynamic::image::image_config::ImageConfig;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::sampler_registry::SamplerType;

pub struct TransientResources {
    pub depth_format: Format,
    
    pub extent: Extent2D,

    pub depth: Arc<ResRef>,
    pub shadow_mask: Arc<ResRef>,
    
    pub depth_image: Arc<ManagedImage>,
    pub shadow_mask_image: Arc<ManagedImage>,
}

impl TransientResources {
    pub fn create(
        instance: &Instance,
        physical_device: PhysicalDevice,
        image_provider: &ResourceProvider<ImageBackend>,
        extent: Extent2D,
    ) -> Result<Self> {
        let extent_3d = Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        };

        let depth_format = find_depth_format(&instance, physical_device)?;
        let depth_aspect = Self::get_depth_aspect_mask(depth_format);
        let depth = image_provider.acquire_sync(ImageConfig::Inbuilt {
            label: String::from("depth"),
            image_description: ImageDescription {
                image_type: ImageType::TYPE_2D,
                format: depth_format,
                extent: extent_3d,
                mip_levels: 1,
                array_layers: 1,
                samples: SampleCountFlags::TYPE_1,
                tiling: ImageTiling::OPTIMAL,
                usage: ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | ImageUsageFlags::SAMPLED,
                sharing_mode: SharingMode::EXCLUSIVE,
            },
            image_view_description: ImageViewDescription {
                image_view_type: ImageViewType::TYPE_2D,
                image_aspect_flags: depth_aspect,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
                layered: false,
            },
            binding: GlobalDescriptorSetBindings::Texture,
            sampler_type: SamplerType::Depth,
            data: None,
        });

        let shadow_mask = image_provider.acquire_sync(ImageConfig::Inbuilt {
            label: String::from("shadow_mask"),
            image_description: ImageDescription {
                image_type: ImageType::TYPE_2D,
                format: Format::R8_UNORM,
                extent: extent_3d,
                mip_levels: 1,
                array_layers: 1,
                samples: SampleCountFlags::TYPE_1,
                tiling: ImageTiling::OPTIMAL,
                usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::COLOR_ATTACHMENT,
                sharing_mode: SharingMode::EXCLUSIVE,
            },
            image_view_description: ImageViewDescription::default_2d_color(),
            binding: GlobalDescriptorSetBindings::Texture,
            sampler_type: SamplerType::ShadowMask,
            data: None,
        });

        let depth_image = image_provider
            .get_resource(depth.id)
            .expect("Depth image must be created after acquire");

        let shadow_mask_image = image_provider
            .get_resource(shadow_mask.id)
            .expect("ShadowMask image must be created after acquire");

        Ok(Self {
            depth_format,

            extent,

            depth,
            shadow_mask,
            
            depth_image,
            shadow_mask_image,
        })
    }

    fn get_depth_aspect_mask(format: Format) -> ImageAspectFlags {
        match format {
            Format::D16_UNORM | Format::D32_SFLOAT | Format::X8_D24_UNORM_PACK32 => {
                ImageAspectFlags::DEPTH
            }
            Format::D16_UNORM_S8_UINT | Format::D24_UNORM_S8_UINT | Format::D32_SFLOAT_S8_UINT => {
                ImageAspectFlags::DEPTH | ImageAspectFlags::STENCIL
            }
            Format::S8_UINT => ImageAspectFlags::STENCIL,
            _ => ImageAspectFlags::DEPTH,
        }
    }

    pub fn destroy(self) {
        drop(self.depth);
        drop(self.shadow_mask);
    }
}
