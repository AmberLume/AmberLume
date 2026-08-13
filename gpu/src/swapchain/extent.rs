use crate::surface_provider::SurfaceProvider;
use anyhow::Result;
use ash::vk::{Extent2D, SurfaceCapabilitiesKHR};
use std::sync::Arc;
use tracing::info;

pub fn create_extent(
    capabilities: &SurfaceCapabilitiesKHR,
    surface_provider: Arc<dyn SurfaceProvider>,
) -> Result<Extent2D> {
    let extent = if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        let (width, height) = surface_provider.size();

        Extent2D {
            width: width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    };

    info!("Extent updated: {:?}", extent);

    Ok(extent)
}
