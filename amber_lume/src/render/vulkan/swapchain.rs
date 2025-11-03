use super::{queue_families::QueueFamilies, vk_surface::VkSurface};
use crate::data::physical_size::PhysicalSize;
use crate::render::vulkan::physical_device_info::PhysicalDeviceInfo;
use crate::render::vulkan::vk_context::VkContext;
use anyhow::Result;
use ash::khr::swapchain::Device;
use ash::vk::{
    ColorSpaceKHR, CompositeAlphaFlagsKHR, Extent2D, Format, ImageAspectFlags,
    ImageSubresourceRange, ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType,
    PresentModeKHR, SharingMode, SwapchainCreateInfoKHR, SwapchainKHR,
};
use ash::vk::{SurfaceCapabilitiesKHR, SurfaceFormatKHR};
use tracing::{debug, trace};

pub struct Swapchain {
    pub loader: Device,

    pub handle: SwapchainKHR,
    pub format: Format,
    pub extent: Extent2D,
    pub image_views: Vec<ImageView>,
}

impl Swapchain {
    pub fn create_loader(vk_context: &VkContext, device: &ash::Device) -> Result<Device> {
        let loader = Device::new(&vk_context.instance, device);

        Ok(loader)
    }

    pub fn create(
        vk_context: &VkContext,
        device: &ash::Device,
        physical_device_info: &PhysicalDeviceInfo,
        vk_surface: &VkSurface,
        queue_families: &QueueFamilies,
        surface_size: &PhysicalSize,
    ) -> Result<Self> {
        let surface_capabilities =
            Self::create_surface_capabilities(vk_context, vk_surface, physical_device_info)?;
        let surface_format =
            Self::get_surface_format(&vk_context, &vk_surface, &physical_device_info)?;
        let present_mode = Self::get_present_mode(&vk_context, &vk_surface, &physical_device_info)?;
        let extent = Self::create_extent(&surface_capabilities, &surface_size)?;

        let swapchain_loader = Self::create_loader(vk_context, &device)?;

        let swapchain = Self::create_swapchain(
            &vk_surface,
            &swapchain_loader,
            &surface_capabilities,
            &queue_families,
            &surface_format,
            extent,
            present_mode,
            None,
        )?;

        let image_views =
            Self::create_image_views(&swapchain_loader, swapchain, surface_format, device)?;

        Ok(Self {
            loader: swapchain_loader,

            handle: swapchain,
            format: surface_format.format,
            extent,
            image_views,
        })
    }

    pub fn recreate(
        &mut self,
        vk_context: &VkContext,
        device: &ash::Device,
        physical_device_info: &PhysicalDeviceInfo,
        vk_surface: &VkSurface,
        queue_families: &QueueFamilies,
        surface_size: &PhysicalSize,
    ) -> Result<()> {
        trace!("Recreating swapchain...");

        let surface_capabilities =
            Self::create_surface_capabilities(vk_context, vk_surface, physical_device_info)?;
        let surface_format =
            Self::get_surface_format(vk_context, vk_surface, physical_device_info)?;
        let present_mode = Self::get_present_mode(vk_context, vk_surface, physical_device_info)?;
        let extent = Self::create_extent(&surface_capabilities, &surface_size)?;

        let swapchain = Self::create_swapchain(
            &vk_surface,
            &self.loader,
            &surface_capabilities,
            &queue_families,
            &surface_format,
            extent,
            present_mode,
            Some(self.handle),
        )?;

        let image_views =
            Self::create_image_views(&self.loader, swapchain, surface_format, device)?;

        self.destroy(device);

        self.image_views = image_views;
        self.handle = swapchain;
        self.format = surface_format.format;
        self.extent = extent;

        Ok(())
    }

    fn get_surface_format(
        vk_context: &VkContext,
        vk_surface: &VkSurface,
        physical_device_info: &PhysicalDeviceInfo,
    ) -> Result<SurfaceFormatKHR> {
        let surface_formats =
            Self::create_surface_formats(vk_context, vk_surface, physical_device_info)?;

        let surface_format = Self::find_surface_format(
            &surface_formats,
            Format::B8G8R8A8_SRGB,
            ColorSpaceKHR::SRGB_NONLINEAR,
        )?;

        Ok(surface_format)
    }

    fn get_present_mode(
        vk_context: &VkContext,
        vk_surface: &VkSurface,
        physical_device_info: &PhysicalDeviceInfo,
    ) -> Result<PresentModeKHR> {
        let surface_present_modes =
            Self::create_surface_present_modes(vk_context, vk_surface, physical_device_info)?;

        let present_mode = Self::find_present_mode(
            surface_present_modes,
            PresentModeKHR::MAILBOX,
            PresentModeKHR::FIFO,
        )?;

        Ok(present_mode)
    }

    fn create_surface_capabilities(
        vk_context: &VkContext,
        vk_surface: &VkSurface,
        physical_device_info: &PhysicalDeviceInfo,
    ) -> Result<SurfaceCapabilitiesKHR> {
        let surface_capabilities = unsafe {
            vk_context
                .surface_loader
                .get_physical_device_surface_capabilities(
                    physical_device_info.handle,
                    vk_surface.surface,
                )?
        };
        debug!("SurfaceCapabilities created: {:?}", surface_capabilities);

        Ok(surface_capabilities)
    }

    fn create_surface_formats(
        vk_context: &VkContext,
        vk_surface: &VkSurface,
        physical_device_info: &PhysicalDeviceInfo,
    ) -> Result<Vec<SurfaceFormatKHR>> {
        let surface_formats = unsafe {
            vk_context
                .surface_loader
                .get_physical_device_surface_formats(
                    physical_device_info.handle,
                    vk_surface.surface,
                )?
        };
        debug!("Supported [SurfaceFormat]: {:?}", surface_formats);

        Ok(surface_formats)
    }

    fn create_surface_present_modes(
        vk_context: &VkContext,
        vk_surface: &VkSurface,
        physical_device_info: &PhysicalDeviceInfo,
    ) -> Result<Vec<PresentModeKHR>> {
        let present_modes = unsafe {
            vk_context
                .surface_loader
                .get_physical_device_surface_present_modes(
                    physical_device_info.handle,
                    vk_surface.surface,
                )?
        };
        debug!("Supported [PresentMode]: {:?}", present_modes);

        Ok(present_modes)
    }

    fn find_surface_format(
        surface_formats: &Vec<SurfaceFormatKHR>,
        desired_format: Format,
        desired_color_space: ColorSpaceKHR,
    ) -> Result<SurfaceFormatKHR> {
        let surface_format = surface_formats
            .iter()
            .copied()
            .find(|f| f.format == desired_format && f.color_space == desired_color_space)
            .unwrap_or(surface_formats[0]);
        debug!("Selected SurfaceFormat: {:?}", surface_format);

        Ok(surface_format)
    }

    fn find_present_mode(
        present_modes: Vec<PresentModeKHR>,
        desired: PresentModeKHR,
        fallback: PresentModeKHR,
    ) -> Result<PresentModeKHR> {
        let present_mode = present_modes
            .into_iter()
            .find(|present_mode| *present_mode == desired)
            .unwrap_or(fallback);
        debug!("Selected PresentMode: {:?}", present_mode);

        Ok(present_mode)
    }

    fn create_extent(
        surface_capabilities: &SurfaceCapabilitiesKHR,
        surface_size: &PhysicalSize,
    ) -> Result<Extent2D> {
        let extent = if surface_capabilities.current_extent.width != u32::MAX {
            surface_capabilities.current_extent
        } else {
            Extent2D {
                width: surface_size.width.clamp(
                    surface_capabilities.min_image_extent.width,
                    surface_capabilities.max_image_extent.width,
                ),
                height: surface_size.height.clamp(
                    surface_capabilities.min_image_extent.height,
                    surface_capabilities.max_image_extent.height,
                ),
            }
        };
        debug!("Extent updated: {:?}", extent);

        Ok(extent)
    }

    fn get_image_count(surface_capabilities: &SurfaceCapabilitiesKHR) -> u32 {
        let mut desired = surface_capabilities.min_image_count + 1;

        let max_image_count = surface_capabilities.max_image_count;

        if max_image_count > 0 && desired > max_image_count {
            desired = max_image_count;
        }
        debug!("Swapchain image count: {}", desired);

        desired
    }

    fn get_sharing_mode_and_queue_families(
        queue_families: &QueueFamilies,
    ) -> Result<(SharingMode, Vec<u32>)> {
        let (sharing, families): (SharingMode, Vec<u32>) =
            if queue_families.graphics != queue_families.present {
                (
                    SharingMode::CONCURRENT,
                    vec![queue_families.graphics, queue_families.present],
                )
            } else {
                (SharingMode::EXCLUSIVE, vec![])
            };
        debug!("SharingMode created: {:?}", sharing);
        debug!("[QueueFamily] created: {:?}", families);

        Ok((sharing, families))
    }

    fn create_swapchain(
        vk_surface: &VkSurface,
        swapchain_loader: &Device,
        surface_capabilities: &SurfaceCapabilitiesKHR,
        queue_families: &QueueFamilies,
        surface_format: &SurfaceFormatKHR,
        extent: Extent2D,
        present_mode: PresentModeKHR,
        old_swapchain: Option<SwapchainKHR>,
    ) -> Result<SwapchainKHR> {
        let image_count = Self::get_image_count(&surface_capabilities);
        let (sharing_mode, queue_family_indices) =
            Self::get_sharing_mode_and_queue_families(queue_families)?;

        let mut swapchain_create_info = SwapchainCreateInfoKHR::default()
            .surface(vk_surface.surface)
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
        debug!("Swapchain created");

        Ok(swapchain)
    }

    fn create_image_views(
        swapchain_loader: &Device,
        swapchain: SwapchainKHR,
        surface_format: SurfaceFormatKHR,
        device: &ash::Device,
    ) -> Result<Vec<ImageView>> {
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        debug!("Swapchain [Image] created");

        let image_views = images
            .into_iter()
            .map(|image| {
                let image_resource_range = ImageSubresourceRange {
                    aspect_mask: ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                };

                let image_create_info = ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .subresource_range(image_resource_range);

                unsafe { device.create_image_view(&image_create_info, None) }
            })
            .collect::<Result<Vec<_>, _>>()?;
        debug!("Swapchain [ImageView] created");

        Ok(image_views)
    }

    pub fn destroy(&self, device: &ash::Device) {
        for &image_view in &self.image_views {
            unsafe { device.destroy_image_view(image_view, None) }
        }
        unsafe {
            self.loader.destroy_swapchain(self.handle, None);
        }
    }
}
