use crate::render::vulkan::factories::image::managed_image::{ImageDescription, ImageViewDescription, ManagedImage};
use crate::render::vulkan::render_pass::depth::depth_format::find_depth_format;
use anyhow::Result;
use ash::Instance;
use ash::vk::{Extent2D, Extent3D, Format, ImageAspectFlags, ImageTiling, ImageType, ImageUsageFlags, ImageViewType, PhysicalDevice, SampleCountFlags, SharingMode};
use tracing::info;
use crate::render::vulkan::factories::image::managed_image_factory::ManagedImageFactory;

pub struct RenderTargets {
    pub depth_image: ManagedImage,
}

impl RenderTargets {
    pub fn create(
        instance: &Instance,
        physical_device: PhysicalDevice,
        image_factory: &ManagedImageFactory,
        extent: Extent2D,
    ) -> Result<Self> {
        let format = find_depth_format(&instance, physical_device)?;

        let depth_vulkan_image = create_depth_image(
            image_factory,
            extent,
            format,
            SampleCountFlags::TYPE_1,
        )?;

        info!("RenderTargets created");

        Ok(Self { depth_image: depth_vulkan_image })
    }

    pub fn destroy(
        self,
        image_factory: &ManagedImageFactory,
    ) -> Result<()> {
        image_factory.destroy_image(self.depth_image)?;

        info!("RenderTargets destroyed");

        Ok(())
    }
}

fn create_depth_image(
    image_factory: &ManagedImageFactory,
    extent: Extent2D,
    format: Format,
    samples: SampleCountFlags,
) -> Result<ManagedImage> {
    let depth_aspect = get_depth_aspect_mask(format);

    let image_description = ImageDescription {
        image_type: ImageType::TYPE_2D,
        format,
        extent: Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        },
        mip_levels: 1,
        array_layers: 1,
        samples,
        tiling: ImageTiling::OPTIMAL,
        usage: ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        sharing_mode: SharingMode::EXCLUSIVE,
    };
    let image_view_description = ImageViewDescription {
        image_view_type: ImageViewType::TYPE_2D,
        image_aspect_flags: depth_aspect,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
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
