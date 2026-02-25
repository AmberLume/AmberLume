use std::sync::Arc;
use anyhow::Result;
use ash::vk::{Extent3D, Format, ImageAspectFlags, ImageSubresourceLayers, ImageTiling, ImageType, ImageUsageFlags, SampleCountFlags, SharingMode};
use crate::render::vulkan::factories::image::managed_image::{ImageDescription, ImageViewDescription, ManagedImage};
use crate::render::vulkan::factories::image::managed_image_factory::ManagedImageFactory;
use crate::render::vulkan::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_manager::IndexManager;

pub struct ImageEntity {
    pub descriptor_index: u32,
    pub managed_image: ManagedImage,
}

pub struct PersistentImages {
    pub white_pixel: ImageEntity,
}

impl PersistentImages {
    pub fn create(
        resource_loader: Arc<ResourceLoader>,
        managed_image_factory: &ManagedImageFactory,
        image_index_manager: &IndexManager,
        format: Format,
        samples: SampleCountFlags,
    ) -> Result<Self> {
        let white_pixel_extent = Extent3D {
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
                extent: white_pixel_extent,
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
            white_pixel_extent,
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

        Ok(Self {
            white_pixel: ImageEntity {
                descriptor_index: white_pixel_resource_id,
                managed_image: white_pixel_managed_image,
            },
        })
    }
    
    pub fn destroy(
        self, 
        managed_image_factory: &ManagedImageFactory,
    ) -> Result<()> {
        managed_image_factory.destroy_image(self.white_pixel.managed_image)?;
        
        Ok(())
    }
}
