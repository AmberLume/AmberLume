use crate::render::vulkan::image::gpu_image::GpuImage;
use anyhow::{Result, bail};
use ash::vk::{
    Extent2D, Format, FormatFeatureFlags, ImageAspectFlags, ImageUsageFlags, PhysicalDevice,
    SampleCountFlags,
};
use ash::{Device, Instance};

const DEPTH_FORMATS: [Format; 4] = [
    Format::D32_SFLOAT,
    Format::D32_SFLOAT_S8_UINT,
    Format::D24_UNORM_S8_UINT,
    Format::D16_UNORM,
];

pub struct DepthImage;

impl DepthImage {
    pub fn create(
        instance: &Instance,
        device: &Device,
        physical_device: PhysicalDevice,

        format: Format,
        extent: Extent2D,
        samples: SampleCountFlags,
    ) -> Result<GpuImage> {
        let aspect = if matches!(format, Format::D32_SFLOAT) {
            ImageAspectFlags::DEPTH
        } else {
            ImageAspectFlags::DEPTH | ImageAspectFlags::STENCIL
        };
        let usage = ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;

        let gpu_image = GpuImage::create(
            instance,
            device,
            physical_device,
            extent,
            format,
            1,
            1,
            samples,
            aspect,
            usage,
        )?;

        Ok(gpu_image)
    }

    pub fn find_depth_format(
        instance: &Instance,
        physical_device: PhysicalDevice,
    ) -> Result<Format> {
        for &format in &DEPTH_FORMATS {
            let properties =
                unsafe { instance.get_physical_device_format_properties(physical_device, format) };

            if properties
                .optimal_tiling_features
                .contains(FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)
            {
                return Ok(format);
            }
        }

        bail!("No supported depth format found");
    }
}
