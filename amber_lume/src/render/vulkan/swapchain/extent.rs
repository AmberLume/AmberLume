use crate::data::physical_size::PhysicalSize;
use anyhow::Result;
use ash::vk::{Extent2D, SurfaceCapabilitiesKHR};
use tracing::info;

pub fn create_extent(
    capabilities: &SurfaceCapabilitiesKHR,
    size: &PhysicalSize,
) -> Result<Extent2D> {
    let extent = if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        Extent2D {
            width: size.width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: size.height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    };

    info!("Extent updated: {:?}", extent);

    Ok(extent)
}
