use anyhow::Result;
use anyhow::bail;
use ash::Instance;
use ash::vk::{Format, FormatFeatureFlags, PhysicalDevice};

const DEPTH_FORMATS: [Format; 4] = [
    Format::D32_SFLOAT,
    Format::D32_SFLOAT_S8_UINT,
    Format::D24_UNORM_S8_UINT,
    Format::D16_UNORM,
];

pub fn find_depth_format(
    instance: &Instance,
    physical_device: PhysicalDevice,
) -> Result<Format> {
    for &format in &DEPTH_FORMATS {
        let properties = unsafe { instance.get_physical_device_format_properties(physical_device, format) };

        if properties.optimal_tiling_features.contains(FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT) {
            return Ok(format);
        }
    }

    bail!("No supported depth format found");
}
