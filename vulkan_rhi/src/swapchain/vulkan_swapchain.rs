use crate::device::vulkan_device::VulkanDevice;
use crate::swapchain::swapchain_config::SwapchainConfig;
use crate::swapchain::swapchain_image::SwapchainImage;
use anyhow::{Result, bail};
use ash::vk::{SharingMode, SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR};
use logger::info;

pub(crate) struct VulkanSwapchain {
    pub swapchain: SwapchainKHR,
    pub config: SwapchainConfig,

    pub images: Vec<SwapchainImage>,
}

impl VulkanSwapchain {
    pub fn create(
        old: Option<&Self>,
        device: &VulkanDevice,
        surface: SurfaceKHR,
        config: SwapchainConfig,
    ) -> Result<Self> {
        if config.extent.width == 0 || config.extent.height == 0 {
            bail!("Window has zero size");
        }

        let mut create_info = SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(config.image_count)
            .image_format(config.format.format)
            .image_color_space(config.format.color_space)
            .image_extent(config.extent)
            .image_array_layers(1)
            .image_usage(config.usage)
            .image_sharing_mode(SharingMode::EXCLUSIVE)
            .pre_transform(config.pre_transform)
            .composite_alpha(config.composite_alpha)
            .present_mode(config.present_mode)
            .clipped(true);

        if let Some(old) = old {
            create_info = create_info.old_swapchain(old.swapchain);
        }

        let swapchain = unsafe { device.swapchain_loader.create_swapchain(&create_info, None)? };

        let images = match SwapchainImage::create(device, swapchain, config.format.format) {
            Ok(images) => images,
            Err(error) => {
                unsafe { device.swapchain_loader.destroy_swapchain(swapchain, None) };

                return Err(error);
            }
        };

        info!(
            "Swapchain created: {}x{}, {:?}, {:?}, {:?}, {} images",
            config.extent.width,
            config.extent.height,
            config.format.format,
            config.format.color_space,
            config.present_mode,
            images.len(),
        );

        Ok(Self {
            swapchain,
            config,

            images,
        })
    }

    pub fn destroy(self, device: &VulkanDevice) {
        for image in self.images {
            image.destroy(device);
        }

        unsafe { device.swapchain_loader.destroy_swapchain(self.swapchain, None) };

        info!("Swapchain destroyed");
    }
}
