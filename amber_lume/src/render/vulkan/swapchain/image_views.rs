use anyhow::Result;
use ash::khr::swapchain::Device;
use ash::vk::{
    Image, ImageAspectFlags, ImageSubresourceRange, ImageView, ImageViewCreateInfo, ImageViewType,
    SurfaceFormatKHR, SwapchainKHR,
};
use tracing::info;

pub fn create_image_views(
    swapchain_loader: &Device,
    swapchain: SwapchainKHR,
    surface_format: SurfaceFormatKHR,
    device: &ash::Device,
) -> Result<(Vec<Image>, Vec<ImageView>)> {
    let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };

    info!("Swapchain [Image] created: {:?}", &images);

    let mut image_views: Vec<ImageView> = Vec::with_capacity(images.len());

    for image in &images {
        let image_resource_range = ImageSubresourceRange {
            aspect_mask: ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };

        let image_create_info = ImageViewCreateInfo::default()
            .image(*image)
            .view_type(ImageViewType::TYPE_2D)
            .format(surface_format.format)
            .subresource_range(image_resource_range);

        let image_view = unsafe { device.create_image_view(&image_create_info, None)? };

        image_views.push(image_view);
    }

    info!("Swapchain [ImageView] created: {:?}", image_views);

    Ok((images, image_views))
}
