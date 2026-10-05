use anyhow::Result;
use dashmap::DashMap;
use ash::vk::{Extent3D, Format, ImageAspectFlags, ImageCreateFlags, ImageSubresourceLayers, ImageTiling, ImageType, ImageUsageFlags, SampleCountFlags, SharingMode};
use std::sync::Arc;
use tracing::info;
use gpu::ImageDescription;
use gpu::ImageViewDescription;
use gpu::ManagedImage;
use gpu::ResourceTransfer;
use resource_residency::ResourceBackend;
use index_allocator::ResourceId;
use gpu::ResourceFactories;
use gpu::BindlessBinding;
use crate::store::image::backend::image_config::ImageConfig;

pub struct ImageBackend {
    resource_factories: Arc<ResourceFactories>,
    resource_transfer: Arc<ResourceTransfer>,

    textures: Arc<BindlessBinding>,

    fallback_image: ManagedImage,

    images: DashMap<ResourceId, ManagedImage>,
}

impl ImageBackend {
    pub(crate) fn new(
        resource_factories: Arc<ResourceFactories>,
        resource_transfer: Arc<ResourceTransfer>,
        textures: Arc<BindlessBinding>,
    ) -> Result<Self> {
        let fallback_image = Self::upload_image(&resource_factories, &resource_transfer, ImageConfig {
            label: "fallback".to_string(),

            image_description: ImageDescription {
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
            },
            image_view_description: ImageViewDescription::default_2d_color(),

            levels: vec![vec![255, 255, 255, 255]],
        })?;

        Ok(Self {
            resource_factories,
            resource_transfer,

            textures,

            fallback_image,

            images: DashMap::new(),
        })
    }

    fn upload_image(
        resource_factories: &ResourceFactories,
        resource_transfer: &ResourceTransfer,
        config: ImageConfig,
    ) -> Result<ManagedImage> {
        let ImageConfig {
            label,

            image_description,
            image_view_description,

            levels,
        } = config;

        let extent = image_description.extent;

        let managed_image = resource_factories.managed_image_factory.allocate(
            &label,
            image_description,
            image_view_description,
        )?;

        let uploaded = levels.iter().enumerate().try_for_each(|(level_index, level_data)| {
            resource_transfer.load_image(
                managed_image.image,
                Extent3D {
                    width: (extent.width >> level_index).max(1),
                    height: (extent.height >> level_index).max(1),
                    depth: 1,
                },
                ImageSubresourceLayers {
                    aspect_mask: ImageAspectFlags::COLOR,
                    mip_level: level_index as u32,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                managed_image.image_description.mip_levels,
                1,
                level_data,
            )
        });

        if let Err(error) = uploaded {
            resource_factories.managed_image_factory.destroy_image(managed_image)?;

            return Err(error);
        }

        Ok(managed_image)
    }
}

impl ResourceBackend for ImageBackend {
    type Config = ImageConfig;
    type Output = ();
    type Statistics = ();

    fn reserve(&self, id: &ResourceId) -> Result<()> {
        self.erase(id)
    }

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output> {
        let managed_image = Self::upload_image(&self.resource_factories, &self.resource_transfer, config)?;

        self.textures.descriptor_set.write(*id, managed_image.image_view);

        info!("Uploaded image: index: {}, label: {}", id.inner, managed_image.label);

        self.images.insert(*id, managed_image);

        Ok(())
    }

    fn erase(&self, id: &ResourceId) -> Result<()> {
        self.textures.descriptor_set.write(*id, self.fallback_image.image_view);

        if let Some((_, image)) = self.images.remove(id) {
            info!("Destroyed image: index: {}, label: {}", id.inner, image.label);

            self.resource_factories.managed_image_factory.destroy_image(image)?;
        }

        Ok(())
    }

    fn statistics(&self) -> Self::Statistics {
        ()
    }

    fn destroy(self) -> Result<()> {
        let Self {
            resource_factories,

            fallback_image,

            images,
            ..
        } = self;

        for (_, image) in images {
            resource_factories.managed_image_factory.destroy_image(image)?;
        }

        resource_factories.managed_image_factory.destroy_image(fallback_image)
    }
}
