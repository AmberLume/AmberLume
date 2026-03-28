use crate::render::factories::image::managed_image::{ImageDescription, ImageViewDescription, ManagedImage};
use crate::render::factories::image::managed_image_factory::ManagedImageFactory;
use anyhow::Result;
use ash::Instance;
use ash::vk::{Extent2D, Extent3D, Format, ImageAspectFlags, ImageTiling, ImageType, ImageUsageFlags, ImageViewType, PhysicalDevice, SampleCountFlags, SharingMode};
use crate::render::render_pass::depth::depth_format::find_depth_format;
use crate::resources::descriptor_index_managers::IndexManagers;
use crate::resources::descriptor_set_manager::GlobalDescriptorSetBindings;
use crate::resources::dynamic::resource_provider::ResourceId;
use crate::resources::persistent::persistent_resources::PersistentResources;

pub struct TransientResources {
    pub depth_descriptor_id: ResourceId,
    pub depth: ManagedImage,

    pub shadow_mask_descriptor_id: ResourceId,
    pub shadow_mask: ManagedImage,
}

impl TransientResources {
    pub fn create(
        instance: &Instance,
        physical_device: PhysicalDevice,
        extent: Extent2D,
        index_managers: &IndexManagers,
        persistent_resources: &PersistentResources,
        image_factory: &ManagedImageFactory,
    ) -> Result<Self> {
        let extent = Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        };

        let depth_descriptor_id = index_managers.texture_descriptors_index_manager.acquire().unwrap();
        let depth = Self::create_depth_image(
            image_factory,
            extent,
            find_depth_format(&instance, physical_device)?,
            SampleCountFlags::TYPE_1,
        )?;
        persistent_resources.descriptor_set_manager.write(
            GlobalDescriptorSetBindings::Texture,
            depth_descriptor_id,
            &depth,
            persistent_resources.samplers.depth,
        );

        let shadow_mask_descriptor_id = index_managers.texture_descriptors_index_manager.acquire().unwrap();
        let shadow_mask = image_factory.allocate(
            "shadow_mask",
            ImageDescription {
                image_type: ImageType::TYPE_2D,
                format: Format::R8_UNORM,
                extent,
                mip_levels: 1,
                array_layers: 1,
                samples: SampleCountFlags::TYPE_1,
                tiling: ImageTiling::OPTIMAL,
                usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::COLOR_ATTACHMENT,
                sharing_mode: SharingMode::EXCLUSIVE,
            },
            ImageViewDescription::default_2d_color(),
        )?;
        persistent_resources.descriptor_set_manager.write(
            GlobalDescriptorSetBindings::Texture,
            shadow_mask_descriptor_id,
            &shadow_mask,
            persistent_resources.samplers.shadow_mask,
        );

        Ok(Self {
            depth_descriptor_id,
            depth,

            shadow_mask_descriptor_id,
            shadow_mask,
        })
    }

    fn create_depth_image(
        image_factory: &ManagedImageFactory,
        extent: Extent3D,
        format: Format,
        samples: SampleCountFlags,
    ) -> Result<ManagedImage> {
        let depth_aspect = Self::get_depth_aspect_mask(format);

        let image_description = ImageDescription {
            image_type: ImageType::TYPE_2D,
            format,
            extent,
            mip_levels: 1,
            array_layers: 1,
            samples,
            tiling: ImageTiling::OPTIMAL,
            usage: ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | ImageUsageFlags::SAMPLED,
            sharing_mode: SharingMode::EXCLUSIVE,
        };
        let image_view_description = ImageViewDescription {
            image_view_type: ImageViewType::TYPE_2D,
            image_aspect_flags: depth_aspect,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,

            layered: false,
        };

        image_factory.allocate(
            "depth",
            image_description,
            image_view_description,
        )
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

    pub fn destroy(
        self,
        index_managers: &IndexManagers,
        image_factory: &ManagedImageFactory,
    ) -> Result<()> {
        image_factory.destroy_image(self.depth)?;
        index_managers.texture_descriptors_index_manager.release(self.depth_descriptor_id);

        image_factory.destroy_image(self.shadow_mask)?;
        index_managers.texture_descriptors_index_manager.release(self.shadow_mask_descriptor_id);

        Ok(())
    }
}
