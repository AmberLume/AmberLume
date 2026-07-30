use crate::queue::queues::Queues;
use crate::surface::render_surface::RenderSurface;
use crate::swapchain::image_count::get_image_count;
use anyhow::Result;
use ash::khr::swapchain::Device;
use ash::vk::{
    CompositeAlphaFlagsKHR, Extent2D, ImageUsageFlags, PresentModeKHR, SurfaceCapabilitiesKHR,
    SurfaceFormatKHR, SwapchainCreateInfoKHR, SwapchainKHR,
};
use tracing::info;

pub fn create_swapchain(
    render_surface: &RenderSurface,
    swapchain_loader: &Device,
    surface_capabilities: &SurfaceCapabilitiesKHR,
    queues: &Queues,
    surface_format: &SurfaceFormatKHR,
    extent: Extent2D,
    present_mode: PresentModeKHR,
    old_swapchain: Option<SwapchainKHR>,
) -> Result<SwapchainKHR> {
    let image_count = get_image_count(&surface_capabilities);
    let sharing_mode = queues.sharing_mode();
    let queue_family_indices = queues.queue_family_indices(sharing_mode);

    info!("Selected SharingMode: {:?}", sharing_mode);
    info!("[QueueFamily] indices: {:?}", queue_family_indices);

    let mut swapchain_create_info = SwapchainCreateInfoKHR::default()
        .surface(render_surface.surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent)
        .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(sharing_mode)
        .image_array_layers(1)
        .queue_family_indices(&queue_family_indices)
        .pre_transform(surface_capabilities.current_transform)
        .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true);

    if let Some(old_swapchain) = old_swapchain {
        swapchain_create_info = swapchain_create_info.old_swapchain(old_swapchain);
    }

    let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };
    info!("Swapchain created");

    Ok(swapchain)
}
