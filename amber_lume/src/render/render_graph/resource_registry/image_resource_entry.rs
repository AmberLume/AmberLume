use std::sync::Arc;
use anyhow::Result;
use ash::vk::{Extent2D, Extent3D, Format, Image, ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, ImageView, SampleCountFlags, SharingMode};
use crate::render::factories::image::image_description::ImageDescription;
use crate::render::factories::image::image_descriptors::ImageDescriptors;
use crate::render::factories::image::managed_image::ManagedImage;
use crate::render::factories::image::managed_image_factory::ManagedImageFactory;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::render::render_graph::virtual_image::physical_image::PhysicalImageDescriptors;
use crate::resources::binding_layout::managed_descriptor_set::ManagedDescriptorSet;
use crate::resources::index::index_manager::IndexManager;
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
        descriptors: ImageDescriptors,
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
            descriptors: ImageDescriptors {
                full: None,
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
        target_extent: Extent2D,
        image_factory: &ManagedImageFactory,
        image_provider: &ResourceProvider<ImageBackend>,
        storage_descriptor_set: &ManagedDescriptorSet,
        storage_index_manager: &IndexManager,
    ) -> Result<()> {
        let Self::Transient { 
            label, 
            blueprint, 
            res_ref, 
            managed,
            descriptors,
        } = self else {
            return Ok(());
        };

        if let Some(old_ids) = descriptors.storage_mips.take() {
            for storage_id in old_ids {
                storage_index_manager.release(storage_id);
            }
        }

        let extent = blueprint.size.resolve(target_extent);

        let image_description = ImageDescription {
            image_type: ImageType::TYPE_2D,
            format: blueprint.format,
            extent: Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            },
            mip_levels: blueprint.image_view_description.level_count,
            array_layers: 1,
            samples: SampleCountFlags::TYPE_1,
            tiling: ImageTiling::OPTIMAL,
            usage: blueprint.usage,
            sharing_mode: SharingMode::EXCLUSIVE,
        };

        if blueprint.sampler.is_some() {
            let new_res_ref = image_provider.acquire_sync(ImageConfig::Inbuilt {
                label: label.to_string(),
                image_description,
                image_view_description: blueprint.image_view_description.clone(),
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

        if blueprint.usage.contains(ImageUsageFlags::STORAGE) {
            let managed = managed.as_ref().expect("image must be built before storage registration");

            let mut new_ids = Vec::with_capacity(managed.storage_mip_views.len());
            for mip_view in &managed.storage_mip_views {
                let storage_id = storage_index_manager.acquire().unwrap();

                storage_descriptor_set.write(storage_id, *mip_view);

                new_ids.push(storage_id);
            }

            descriptors.storage_mips = Some(new_ids);
        }

        Ok(())
    }
}
