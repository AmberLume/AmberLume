use crate::device::vulkan_device::VulkanDevice;
use anyhow::Result;
use ash::vk::{
    Format, Image, ImageAspectFlags, ImageSubresourceRange, ImageView, ImageViewCreateInfo, ImageViewType, SwapchainKHR,
};

pub(crate) struct SwapchainImage {
    pub image: Image,
    pub image_view: ImageView,
}

impl SwapchainImage {
    const SUBRESOURCE_RANGE: ImageSubresourceRange = ImageSubresourceRange {
        aspect_mask: ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
    };

    pub fn create(device: &VulkanDevice, swapchain: SwapchainKHR, format: Format) -> Result<Vec<Self>> {
        let images = unsafe { device.swapchain_loader.get_swapchain_images(swapchain)? };

        let mut swapchain_images = Vec::with_capacity(images.len());

        for image in images {
            let create_info = ImageViewCreateInfo::default()
                .image(image)
                .view_type(ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(Self::SUBRESOURCE_RANGE);

            match unsafe { device.device.create_image_view(&create_info, None) } {
                Ok(image_view) => swapchain_images.push(Self { image, image_view }),
                Err(error) => {
                    for swapchain_image in swapchain_images {
                        swapchain_image.destroy(device);
                    }

                    return Err(error.into());
                }
            }
        }

        Ok(swapchain_images)
    }

    pub fn destroy(self, device: &VulkanDevice) {
        unsafe { device.device.destroy_image_view(self.image_view, None) };
    }
}
