use anyhow::Result;
use ash::khr::swapchain::Device;
use ash::vk;
use tracing::{debug, info};
use vk::{
    ColorSpaceKHR, CompositeAlphaFlagsKHR, Extent2D, Format, ImageAspectFlags,
    ImageSubresourceRange, ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType,
    PresentModeKHR, SharingMode, SwapchainCreateInfoKHR, SwapchainKHR,
};
use winit::window::Window;

use super::{
    instance_surface::InstanceSurface, logical_device::LogicalDevice, queue_families::QueueFamilies,
};

pub struct Swapchain {
    pub loader: Device,
    pub handle: SwapchainKHR,
    pub format: Format,
    pub extent: Extent2D,
    pub image_views: Vec<ImageView>,
}

impl Swapchain {
    pub fn create(
        instance_surface: &InstanceSurface,
        logical_device: &LogicalDevice,
        queue_families: &QueueFamilies,
        window: &Window,
    ) -> Result<Self> {
        debug!("Creating PhysicalDevice surface capabilities...");
        let surface_capabilities = unsafe {
            instance_surface
                .surface_loader
                .get_physical_device_surface_capabilities(
                    logical_device.physical_device,
                    instance_surface.surface,
                )?
        };
        debug!(
            "PhysicalDevice surface capabilities created: {:?}",
            surface_capabilities
        );

        debug!("Creating PhysicalDevice surface formats...");
        let surface_formats = unsafe {
            instance_surface
                .surface_loader
                .get_physical_device_surface_formats(
                    logical_device.physical_device,
                    instance_surface.surface,
                )?
        };
        debug!(
            "PhysicalDevice surface formats created: {:?}",
            surface_formats
        );

        debug!("Creating PhysicalDevice PresentModes...");
        let present_modes = unsafe {
            instance_surface
                .surface_loader
                .get_physical_device_surface_present_modes(
                    logical_device.physical_device,
                    instance_surface.surface,
                )?
        };
        debug!("PhysicalDevice PresentModes created: {:?}", present_modes);

        debug!("Searching for SurfaceFormats...");
        let surface_format = surface_formats
            .iter()
            .copied()
            .find(|f| {
                f.format == Format::B8G8R8A8_SRGB && f.color_space == ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(surface_formats[0]);
        debug!("SurfaceFormats found: {:?}", surface_format);

        debug!("Searching for surface PresentMode...");
        let present_mode = present_modes
            .into_iter()
            .find(|m| *m == PresentModeKHR::MAILBOX)
            .unwrap_or(PresentModeKHR::FIFO);
        debug!("Surface PresentMode found: {:?}", present_mode);

        let extent = if surface_capabilities.current_extent.width != u32::MAX {
            surface_capabilities.current_extent
        } else {
            let size = window.inner_size();
            Extent2D {
                width: size.width.clamp(
                    surface_capabilities.min_image_extent.width,
                    surface_capabilities.max_image_extent.width,
                ),
                height: size.height.clamp(
                    surface_capabilities.min_image_extent.height,
                    surface_capabilities.max_image_extent.height,
                ),
            }
        };
        debug!("Current extent size: {:?}", extent);

        let mut image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0
            && image_count > surface_capabilities.max_image_count
        {
            image_count = surface_capabilities.max_image_count;
        }
        debug!("Swapchain image count: {}", image_count);

        let (sharing, families): (SharingMode, Vec<u32>) =
            if queue_families.graphics != queue_families.present {
                (
                    SharingMode::CONCURRENT,
                    vec![queue_families.graphics, queue_families.present],
                )
            } else {
                (SharingMode::EXCLUSIVE, vec![])
            };
        debug!("Swapchain sharing mode: {:?}", sharing);
        debug!("Swapchain families: {:?}", families);

        let loader = Device::new(&instance_surface.instance, &logical_device.device);
        let swapchain_create_info = SwapchainCreateInfoKHR::default()
            .surface(instance_surface.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(sharing)
            .image_array_layers(1)
            .queue_family_indices(&families)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);
        debug!("Creating Swapchain...");
        let swapchain = unsafe { loader.create_swapchain(&swapchain_create_info, None)? };
        debug!("Swapchain created");

        debug!("Creating Swapchain images...");
        let images = unsafe { loader.get_swapchain_images(swapchain)? };
        debug!("Swapchain images created");

        debug!("Creating Swapchain ImageViews...");
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

                unsafe {
                    logical_device
                        .device
                        .create_image_view(&image_create_info, None)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        debug!("Swapchain ImageViews created");

        Ok(Self {
            loader,
            handle: swapchain,
            format: surface_format.format,
            extent,
            image_views,
        })
    }

    pub fn destroy(&self, logical_device: &LogicalDevice) {
        unsafe {
            for &image_view in &self.image_views {
                logical_device.device.destroy_image_view(image_view, None);
            }
            self.loader.destroy_swapchain(self.handle, None);
        }
    }
}
