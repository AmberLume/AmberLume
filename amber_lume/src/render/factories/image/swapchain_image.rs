use crate::render::factories::image::image_utils::{
    create_image_subresource_range, create_image_view,
};
use ash::khr::swapchain::Device as SwapchainDevice;
use anyhow::Result;
use ash::Device;
use ash::vk::{Extent2D, Format, Image, ImageSubresourceRange, ImageView, SwapchainKHR};
use tracing::info;
use crate::render::factories::image::image_view_description::ImageViewDescription;

#[derive(Debug)]
pub struct SwapchainImage {
    pub label: String,

    pub format: Format,
    pub extent: Extent2D,

    pub image: Image,
    pub image_view: ImageView,

    pub image_subresource_range: ImageSubresourceRange,
}

impl SwapchainImage {
    pub fn create(
        device: &Device,
        swapchain_loader: &SwapchainDevice,
        swapchain: SwapchainKHR,
        format: Format,
        extent: Extent2D,
    ) -> Result<Vec<Self>> {
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        
        let image_view_description = ImageViewDescription::default_2d_color();
        let image_subresource_range = create_image_subresource_range(&image_view_description);

        let images = images
            .into_iter()
            .enumerate()
            .map(|(index, image)| {
                let image_view = create_image_view(
                    &device,
                    image,
                    format,
                    &image_view_description,
                    image_subresource_range,
                )
                .unwrap();

                info!("SwapchainImage '{}' created", index);

                Self {
                    label: format!("swapchain_image_{}", index),

                    format,
                    extent,

                    image,
                    image_view,
                    
                    image_subresource_range,
                }
            })
            .collect::<Vec<_>>();

        Ok(images)
    }

    pub fn destroy(&self, device: &Device) -> Result<()> {
        unsafe { device.destroy_image_view(self.image_view, None) };

        info!("SwapchainImage '{}' destroyed", self.label);

        Ok(())
    }
}
