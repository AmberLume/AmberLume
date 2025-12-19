use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::vulkan_context::VulkanContext;
use anyhow::Result;
use anyhow::bail;
use ash::vk::{Format, FormatFeatureFlags};

const DEPTH_FORMATS: [Format; 4] = [
    Format::D32_SFLOAT,
    Format::D32_SFLOAT_S8_UINT,
    Format::D24_UNORM_S8_UINT,
    Format::D16_UNORM,
];

pub fn find_depth_format(
    vulkan_context: &VulkanContext,
    device_context: &DeviceContext,
) -> Result<Format> {
    for &format in &DEPTH_FORMATS {
        let properties = unsafe {
            vulkan_context
                .instance
                .get_physical_device_format_properties(
                    device_context.physical_device_info.handle,
                    format,
                )
        };

        if properties
            .optimal_tiling_features
            .contains(FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)
        {
            return Ok(format);
        }
    }

    bail!("No supported depth format found");
}
