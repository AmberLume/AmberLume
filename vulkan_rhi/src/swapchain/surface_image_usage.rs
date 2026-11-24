use anyhow::{Result, bail};
use ash::vk::{ImageUsageFlags, SurfaceCapabilitiesKHR};
use explicit_rhi::ImageUsage;

pub(crate) struct SurfaceImageUsage;

impl SurfaceImageUsage {
    pub fn select(capabilities: &SurfaceCapabilitiesKHR, usage: ImageUsage) -> Result<ImageUsageFlags> {
        let mut flags = ImageUsageFlags::empty();

        if usage.color_attachment {
            flags |= ImageUsageFlags::COLOR_ATTACHMENT;
        }

        if flags.is_empty() {
            bail!("No image usage requested");
        }

        if !capabilities.supported_usage_flags.contains(flags) {
            bail!("Image usage {:?} is not supported by the surface, supported: {:?}", flags, capabilities.supported_usage_flags);
        }

        Ok(flags)
    }
}
