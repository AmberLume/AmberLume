use anyhow::Result;
use ash::vk::{Extent3D, Format, ImageAspectFlags, ImageTiling, ImageType, ImageUsageFlags, ImageViewType, SampleCountFlags, SharingMode};
use crate::render::vulkan::factories::image::managed_image::{ImageDescription, ImageViewDescription, ManagedImage};
use crate::render::vulkan::factories::image::managed_image_factory::ManagedImageFactory;
use crate::resources::descriptor_index_managers::IndexManagers;
use crate::resources::dynamic::resource_provider::ResourceId;
use crate::resources::persistent::persistent_descriptor_set_layouts::GlobalDescriptorSetBindings;
use crate::resources::persistent::persistent_descriptor_sets::PersistentDescriptorSets;
use crate::resources::persistent::persistent_samplers::PersistentSamplers;

pub struct PersistentShadows {
    pub global_shadow_descriptor_id: ResourceId,
    pub global_texture_descriptor_id: ResourceId,
    pub global_shadow: ManagedImage,
}

impl PersistentShadows {
    pub fn create(
        persistent_descriptor_sets: &PersistentDescriptorSets,
        persistent_samplers: &PersistentSamplers,
        managed_image_factory: &ManagedImageFactory,
        index_managers: &IndexManagers,
    ) -> Result<Self> {
        let global_shadow_descriptor_id = index_managers.shadow_descriptors_index_manager.acquire().unwrap();
        let global_texture_descriptor_id = index_managers.image_descriptors_index_manager.acquire().unwrap();
        let global_shadow = managed_image_factory.allocate(
            "global_shadow",
            ImageDescription {
                image_type: ImageType::TYPE_2D,
                format: Format::D32_SFLOAT,
                extent: Extent3D {
                    width: 4096,
                    height: 4096,
                    depth: 1,
                },
                mip_levels: 1,
                array_layers: 1,
                samples: SampleCountFlags::TYPE_1,
                tiling: ImageTiling::OPTIMAL,
                usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                sharing_mode: SharingMode::EXCLUSIVE,
            },
            ImageViewDescription {
                image_view_type: ImageViewType::TYPE_2D,
                image_aspect_flags: ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        )?;
        persistent_descriptor_sets.global.bind_image(
            global_texture_descriptor_id,
            GlobalDescriptorSetBindings::Texture as u32,
            &global_shadow,
            persistent_samplers.shadow,
        );
        persistent_descriptor_sets.global.bind_image(
            global_shadow_descriptor_id,
            GlobalDescriptorSetBindings::Shadow as u32,
            &global_shadow,
            persistent_samplers.shadow,
        );

        Ok(Self {
            global_texture_descriptor_id,
            global_shadow_descriptor_id,
            global_shadow,
        })
    }

    pub fn destroy(
        self,
        managed_image_factory: &ManagedImageFactory,
    ) -> Result<()> {
        managed_image_factory.destroy_image(self.global_shadow)?;

        Ok(())
    }
}
