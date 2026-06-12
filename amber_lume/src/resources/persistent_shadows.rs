use anyhow::Result;
use ash::vk::{Extent3D, Format, ImageTiling, ImageType, ImageUsageFlags, SampleCountFlags, SharingMode};
use crate::limits::{ShadowMapFormat, ShadowMapParams};
use crate::render::factories::image::managed_image::ManagedImage;
use crate::render::factories::image::image_description::ImageDescription;
use crate::render::factories::image::image_view_description::ImageViewDescription;
use crate::render::factories::image::managed_image_factory::ManagedImageFactory;
use crate::resources::bindless::bindless_binding::BindlessBinding;
use crate::resources::bindless::bindless_image::BindlessImage;

pub struct PersistentShadows {
    pub global_shadow_array_descriptor: BindlessImage,
    pub global_shadow_array: ManagedImage,
}

impl PersistentShadows {
    pub fn create(
        shadow_binding: &BindlessBinding,
        managed_image_factory: &ManagedImageFactory,
        limits: &ShadowMapParams,
    ) -> Result<Self> {
        let global_shadow_array = managed_image_factory.allocate(
            "global_shadow_array",
            ImageDescription {
                image_type: ImageType::TYPE_2D,
                format: match limits.format {
                    ShadowMapFormat::D16 => Format::D16_UNORM,
                    ShadowMapFormat::D32 => Format::D32_SFLOAT,
                },
                extent: Extent3D {
                    width: limits.resolution,
                    height: limits.resolution,
                    depth: 1,
                },
                mip_levels: 1,
                array_layers: limits.cascade_count,
                samples: SampleCountFlags::TYPE_1,
                tiling: ImageTiling::OPTIMAL,
                usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                sharing_mode: SharingMode::EXCLUSIVE,
            },
            ImageViewDescription::default_2d_array_depth(limits.cascade_count),
        )?;
        let global_shadow_array_descriptor = shadow_binding
            .acquire_image(global_shadow_array.image_view)
            .expect("shadow array descriptor capacity exceeded");

        Ok(Self {
            global_shadow_array_descriptor,
            global_shadow_array,
        })
    }

    pub fn destroy(
        self,
        managed_image_factory: &ManagedImageFactory,
    ) -> Result<()> {
        managed_image_factory.destroy_image(self.global_shadow_array)?;

        Ok(())
    }
}
