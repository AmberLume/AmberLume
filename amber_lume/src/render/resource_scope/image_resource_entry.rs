use std::sync::Arc;
use anyhow::Result;
use ash::vk::{Extent2D, Extent3D, Format, Image, ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, ImageView, SampleCountFlags, SharingMode};
use gpu::ImageDescription;
use gpu::ImageDescriptors;
use gpu::ManagedImage;
use gpu::ManagedImageFactory;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use gpu::BindlessBinding;
use gpu::BindlessImage;
use gpu::ArcUnwrapOrErr;

pub enum ImageResourceEntry {
    Transient {
        label: &'static str,
        blueprint: ImageBlueprint,
        managed: Option<Arc<ManagedImage>>,
        descriptors: ImageDescriptors,
    },
    Imported {
        image: Image,
        image_view: ImageView,
        extent: Extent2D,
        format: Format,
        subresource_range: ImageSubresourceRange,
        descriptor: Option<BindlessImage>,
    },
}

impl ImageResourceEntry {
    pub fn transient(label: &'static str, blueprint: ImageBlueprint) -> Self {
        Self::Transient {
            label,
            blueprint,
            managed: None,
            descriptors: ImageDescriptors {
                view: None,
                sampled_mips: None,
                storage_mips: None,
            },
        }
    }

    pub fn imported(
        image: Image,
        image_view: ImageView,
        extent: Extent2D,
        format: Format,
        subresource_range: ImageSubresourceRange,
        descriptor: Option<BindlessImage>,
    ) -> Self {
        Self::Imported {
            image,
            image_view,
            extent,
            format,
            subresource_range,
            descriptor,
        }
    }

    pub fn build(
        &mut self,
        target_extent: Extent2D,
        render_extent: Extent2D,
        image_factory: &ManagedImageFactory,
        graph_textures: &BindlessBinding,
        storage_binding: &BindlessBinding,
    ) -> Result<()> {
        let Self::Transient {
            label,
            blueprint,
            managed,
            descriptors,
        } = self else {
            return Ok(());
        };

        descriptors.view = None;
        descriptors.sampled_mips = None;
        descriptors.storage_mips = None;

        let extent = blueprint.size.resolve(target_extent, render_extent);

        let image_description = ImageDescription {
            image_type: ImageType::TYPE_2D,
            format: blueprint.format,
            extent: Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            },
            mip_levels: blueprint.image_view_description.level_count,
            array_layers: blueprint.array_layers,
            samples: SampleCountFlags::TYPE_1,
            tiling: ImageTiling::OPTIMAL,
            usage: blueprint.usage,
            sharing_mode: SharingMode::EXCLUSIVE,
            flags: blueprint.flags,
        };

        if let Some(old) = managed.take() {
            image_factory.destroy_image(old.try_unwrap()?)?;
        }

        let managed_image = image_factory.allocate(
            label,
            image_description,
            blueprint.image_view_description,
        )?;

        *managed = Some(Arc::new(managed_image));
        let managed = managed.as_ref().expect("image must be built");

        if blueprint.sampled {
            descriptors.view = graph_textures.acquire_image(managed.image_view);
        }

        if blueprint.sampled && image_description.mip_levels > 1 {
            descriptors.sampled_mips = Some(
                graph_textures
                    .acquire_image_array(&managed.mip_views)
                    .expect("sampled descriptor capacity exceeded"),
            );
        }

        if blueprint.usage.contains(ImageUsageFlags::STORAGE) {
            descriptors.storage_mips = Some(
                storage_binding
                    .acquire_image_array(&managed.mip_views)
                    .expect("storage descriptor capacity exceeded"),
            );
        }

        Ok(())
    }
}
