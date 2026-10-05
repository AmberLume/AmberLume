use crate::store::image::backend::image_backend::ImageBackend;
use crate::store::image::backend::image_config::ImageConfig;
use crate::store::image::loaders::texture_format::TextureFormat;
use anyhow::Result;
use ash::vk::{Extent3D, ImageAspectFlags, ImageCreateFlags, ImageTiling, ImageType, ImageUsageFlags, ImageViewType, SampleCountFlags, SharingMode};
use asset_codec::TextureData;
use gpu::ImageDescription;
use gpu::ImageViewDescription;
use resource_reader::ResourceReader;
use resource_residency::ResRef;
use resource_residency::ResourceProvider;
use std::sync::Arc;

pub struct ImageLoader {
    resource_reader: Arc<dyn ResourceReader>,

    texture_format: TextureFormat,

    image_provider: Arc<ResourceProvider<ImageBackend>>,
}

impl ImageLoader {
    pub fn new(
        resource_reader: Arc<dyn ResourceReader>,
        texture_format: TextureFormat,
        image_provider: Arc<ResourceProvider<ImageBackend>>,
    ) -> Self {
        Self {
            resource_reader,

            texture_format,

            image_provider,
        }
    }

    pub fn load(self: &Arc<Self>, resource_key: &str) -> Result<Arc<ResRef>> {
        let loader = self.clone();
        let key = resource_key.to_string();

        self.image_provider.get_or_load(&resource_key, move || loader.read(&key))
    }

    fn read(&self, resource_key: &str) -> Result<ImageConfig> {
        let image_bytes = self.resource_reader.get_resource(resource_key)?;

        let texture_data = TextureData::decode(
            image_bytes,
            self.texture_format.block_format,
        )?;

        let format = if texture_data.is_srgb {
            self.texture_format.color_srgb
        } else {
            self.texture_format.linear
        };

        Ok(ImageConfig {
            label: resource_key.to_string(),

            image_description: ImageDescription {
                image_type: ImageType::TYPE_2D,
                format,
                extent: Extent3D {
                    width: texture_data.width,
                    height: texture_data.height,
                    depth: 1,
                },
                mip_levels: texture_data.mip_levels,
                array_layers: 1,
                samples: SampleCountFlags::TYPE_1,
                tiling: ImageTiling::OPTIMAL,
                usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
                sharing_mode: SharingMode::EXCLUSIVE,
                flags: ImageCreateFlags::empty(),
            },
            image_view_description: ImageViewDescription {
                image_view_type: ImageViewType::TYPE_2D,
                image_aspect_flags: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: texture_data.mip_levels,
                base_array_layer: 0,
                layer_count: 1,
            },

            levels: texture_data.levels,
        })
    }
}
