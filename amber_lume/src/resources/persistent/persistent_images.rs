use std::sync::Arc;
use anyhow::Result;
use ash::vk::{Extent3D, Format, ImageAspectFlags, ImageSubresourceLayers, ImageTiling, ImageType, ImageUsageFlags, SampleCountFlags, SharingMode};
use crate::render::factories::image::managed_image::{ImageDescription, ImageViewDescription, ManagedImage};
use crate::render::factories::image::managed_image_factory::ManagedImageFactory;
use crate::render::resources::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_manager::IndexManager;
use crate::resources::persistent::persistent_descriptor_set_layouts::GlobalDescriptorSetBindings;
use crate::resources::persistent::persistent_descriptor_sets::PersistentDescriptorSets;
use crate::resources::persistent::persistent_samplers::PersistentSamplers;

pub struct ImageEntity {
    pub descriptor_index: u32,
    pub managed_image: ManagedImage,
}

pub struct PersistentImages {
    pub white_pixel: ImageEntity,
    pub default_normal: ImageEntity,
    pub default_occlusion_roughness_metallic: ImageEntity,
}

impl PersistentImages {
    pub fn create(
        resource_loader: Arc<ResourceLoader>,
        managed_image_factory: &ManagedImageFactory,
        image_index_manager: &IndexManager,
        descriptor_sets: &PersistentDescriptorSets,
        samplers: &PersistentSamplers,
        format: Format,
        samples: SampleCountFlags,
    ) -> Result<Self> {
        let pixel_extent = Extent3D {
            width: 1,
            height: 1,
            depth: 1,
        };

        let white_pixel_resource_id = image_index_manager.acquire().unwrap();
        let white_pixel_managed_image = managed_image_factory.allocate(
            "white_pixel",
            ImageDescription {
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
            ImageViewDescription::default_2d_color(),
        )?;
        resource_loader.load_image(
            white_pixel_managed_image.image,
            pixel_extent,
            ImageSubresourceLayers {
                aspect_mask: ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            1,
            1,
            &[255, 255, 255, 255]
        )?;
        descriptor_sets.global.bind_image(
            white_pixel_resource_id,
            GlobalDescriptorSetBindings::Texture as u32,
            &white_pixel_managed_image,
            samplers.linear_repeat,
        );

        let default_normal_resource_id = image_index_manager.acquire().unwrap();
        let default_normal_managed_image = managed_image_factory.allocate(
            "default_normal",
            ImageDescription {
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
            ImageViewDescription::default_2d_color(),
        )?;
        resource_loader.load_image(
            default_normal_managed_image.image,
            pixel_extent,
            ImageSubresourceLayers {
                aspect_mask: ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            1,
            1,
            &[128, 128, 255, 0]
        )?;
        descriptor_sets.global.bind_image(
            default_normal_resource_id,
            GlobalDescriptorSetBindings::Texture as u32,
            &default_normal_managed_image,
            samplers.linear_repeat,
        );

        let default_occlusion_roughness_metallic_resource_id = image_index_manager.acquire().unwrap();
        let default_occlusion_roughness_metallic_managed_image = managed_image_factory.allocate(
            "default_occlusion_roughness_metallic",
            ImageDescription {
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
            ImageViewDescription::default_2d_color(),
        )?;
        resource_loader.load_image(
            default_occlusion_roughness_metallic_managed_image.image,
            pixel_extent,
            ImageSubresourceLayers {
                aspect_mask: ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            1,
            1,
            &[255, 128, 0, 0]
        )?;
        descriptor_sets.global.bind_image(
            default_occlusion_roughness_metallic_resource_id,
            GlobalDescriptorSetBindings::Texture as u32,
            &default_occlusion_roughness_metallic_managed_image,
            samplers.linear_repeat,
        );

        Ok(Self {
            white_pixel: ImageEntity {
                descriptor_index: white_pixel_resource_id,
                managed_image: white_pixel_managed_image,
            },
            default_normal: ImageEntity {
                descriptor_index: default_normal_resource_id,
                managed_image: default_normal_managed_image,
            },
            default_occlusion_roughness_metallic: ImageEntity {
                descriptor_index: default_occlusion_roughness_metallic_resource_id,
                managed_image: default_occlusion_roughness_metallic_managed_image,
            },
        })
    }
    
    pub fn destroy(
        self, 
        managed_image_factory: &ManagedImageFactory,
    ) -> Result<()> {
        managed_image_factory.destroy_image(self.default_occlusion_roughness_metallic.managed_image)?;
        managed_image_factory.destroy_image(self.default_normal.managed_image)?;
        managed_image_factory.destroy_image(self.white_pixel.managed_image)?;
        
        Ok(())
    }
}
