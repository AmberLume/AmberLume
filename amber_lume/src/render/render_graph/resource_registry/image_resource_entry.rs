use std::sync::Arc;
use anyhow::Result;
use ash::vk::{Extent2D, Extent3D, Format, Image, ImageSubresourceRange, ImageTiling, ImageType, ImageView, SampleCountFlags, SharingMode};
use crate::render::factories::image::image_description::ImageDescription;
use crate::render::factories::image::managed_image::ManagedImage;
use crate::render::factories::image::managed_image_factory::ManagedImageFactory;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::resources::store::providers::image::image_backend::ImageBackend;
use crate::resources::store::providers::image::image_config::ImageConfig;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::utils::arc_utils::ArcUnwrapOrErr;

pub enum ImageResourceEntry {
    Transient {
        label: &'static str,
        blueprint: ImageBlueprint,
        res_ref: Option<Arc<ResRef>>,
        managed: Option<Arc<ManagedImage>>,
    },
    Imported {
        image: Image,
        image_view: ImageView,
        extent: Extent2D,
        format: Format,
        subresource_range: ImageSubresourceRange,
        descriptor_id: Option<u32>,
    },
}

impl ImageResourceEntry {
    pub fn transient(label: &'static str, blueprint: ImageBlueprint) -> Self {
        Self::Transient {
            label,
            blueprint,
            managed: None,
            res_ref: None,
        }
    }

    pub fn imported(
        image: Image,
        image_view: ImageView,
        extent: Extent2D,
        format: Format,
        subresource_range: ImageSubresourceRange,
        descriptor_id: Option<u32>,
    ) -> Self {
        Self::Imported {
            image,
            image_view,
            extent,
            format,
            subresource_range,
            descriptor_id,
        }
    }

    pub fn build(
        &mut self,
        swapchain_extent: Extent2D,
        image_factory: &ManagedImageFactory,
        image_provider: &ResourceProvider<ImageBackend>,
    ) -> Result<()> {
        let Self::Transient { label, blueprint, res_ref, managed } = self else {
            return Ok(());
        };

        let extent = blueprint.size.resolve(swapchain_extent);

        let image_description = ImageDescription {
            image_type: ImageType::TYPE_2D,
            format: blueprint.format,
            extent: Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            samples: SampleCountFlags::TYPE_1,
            tiling: ImageTiling::OPTIMAL,
            usage: blueprint.usage,
            sharing_mode: SharingMode::EXCLUSIVE,
        };

        if let Some((binding, sampler_type)) = blueprint.descriptor {
            let new_res_ref = image_provider.acquire_sync(ImageConfig::Inbuilt {
                label: label.to_string(),
                image_description,
                image_view_description: blueprint.image_view_description.clone(),
                binding,
                sampler_type,
                data: None,
            });

            let new_managed = image_provider
                .get_resource(new_res_ref.id)
                .expect("Image must be available after acquire");

            *managed = Some(new_managed);
            *res_ref = Some(new_res_ref);
        } else {
            if let Some(old) = managed.take() {
                image_factory.destroy_image(old.try_unwrap()?)?;
            }

            let managed_image = image_factory.allocate(
                label,
                image_description,
                blueprint.image_view_description,
            )?;

            *managed = Some(Arc::new(managed_image));
        }

        Ok(())
    }
}
